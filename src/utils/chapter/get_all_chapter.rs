use std::task::Poll;

use crate::settings::file_history::{History, IsIn};

use super::GetAllChapter;
use futures::StreamExt;
use tokio::sync::OwnedRwLockReadGuard;
use tokio_stream::Stream;

pub struct OnlyFails<T>
where
    T: Stream<Item = String> + Unpin,
{
    inner: T,
}

impl<T> OnlyFails<T> where T: Stream<Item = String> + Unpin {
    pub fn new(inner : T) -> Self{
        Self { inner }
    }
}

impl<T> Stream for OnlyFails<T> where T: Stream<Item = String> + Unpin {
    type Item = String;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx)
    }
}

pub struct NotIncludeFails<T>
where
    T: Stream<Item = String> + Unpin,
{
    all_chapter: T,
    history: OwnedRwLockReadGuard<History>
}

impl<T> NotIncludeFails<T> where T: Stream<Item = String> + Unpin  {
    pub fn new(all_chapter: T, history: OwnedRwLockReadGuard<History>) -> Self{
        Self { all_chapter, history }
    }
}

impl<T> Stream for NotIncludeFails<T> where T: Stream<Item = String> + Unpin  {
    type Item = String;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        if let Poll::Ready(getted) = self.all_chapter.poll_next_unpin(cx) {
            if let Some(id) = getted {
                if let Ok(uuid) = <uuid::Uuid as TryFrom<&str>>::try_from(id.clone().as_str()){
                    if self.history.is_in(uuid).is_none() {
                        Poll::Ready(Some(id))
                    }else {
                        Poll::Pending
                    }
                }else {
                    Poll::Pending
                }
            }else {
                Poll::Ready(None)
            }
        }else {
            Poll::Pending
        }
    }
    
}

pub struct AsyncGetAllChapter<C, H>
where
    C: Stream<Item = String> + std::marker::Unpin,
    H: Stream<Item = String> + std::marker::Unpin,
{
    pub only_fails: OnlyFails<H>,
    pub parameters: GetAllChapter,
    pub not_fails: NotIncludeFails<C>
}

impl<C, H> Stream for AsyncGetAllChapter<C, H>
where
    C: Stream<Item = String> + std::marker::Unpin,
    H: Stream<Item = String> + std::marker::Unpin,  
{
    type Item = String;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        if self.parameters.only_fails {
            self.only_fails.poll_next_unpin(cx)
        }else if !self.parameters.include_fails {
            self.not_fails.poll_next_unpin(cx)
        }else{
            self.not_fails.all_chapter.poll_next_unpin(cx)
        }
    }
}