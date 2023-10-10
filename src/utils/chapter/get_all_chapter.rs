use std::task::Poll;

use crate::settings::file_history::{History, IsIn};

use super::GetAllChapter;
use futures::StreamExt;
use log::info;
use tokio_stream::Stream;

pub struct OnlyFails<T>
where
    T: Stream<Item = String> + Unpin,
{
    inner: T,
}

impl<T> OnlyFails<T>
where
    T: Stream<Item = String> + Unpin,
{
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> Stream for OnlyFails<T>
where
    T: Stream<Item = String> + Unpin,
{
    type Item = String;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx)
    }
}

pub struct NotIncludeFails<T>
where
    T: Stream<Item = String> + Unpin,
{
    all_chapter: T,
    history: Vec<String>,
}

impl IsIn<String> for Vec<String> {
    type Output = Option<usize>;
    fn is_in(&self, to_use: String) -> Self::Output {
        let pos = self.iter().position(|d| d.cmp(&to_use).is_eq());
        info!("{:?}", pos);
        pos
    }
}

impl<T> NotIncludeFails<T>
where
    T: Stream<Item = String> + Unpin,
{
    pub fn new(all_chapter: T, history: History) -> Self {
        Self {
            all_chapter,
            history: history
                .get_history_list()
                .iter()
                .map(|d| d.to_string())
                .collect(),
        }
    }
}

impl<T> Stream for NotIncludeFails<T>
where
    T: Stream<Item = String> + Unpin,
{
    type Item = (String, bool);

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Poll::Ready(getted) = self.all_chapter.poll_next_unpin(cx) {
            if let Some(id) = getted {
                Poll::Ready(Some((id, self.history.is_in(id.clone()).is_none())))
            } else {
                log::info!("Exited");
                Poll::Ready(None)
            }
        } else {
            log::info!("Pending 1");
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
    pub not_fails: NotIncludeFails<C>,
}

impl<C, H> Stream for AsyncGetAllChapter<C, H>
where
    C: Stream<Item = String> + std::marker::Unpin,
    H: Stream<Item = String> + std::marker::Unpin,
{
    type Item = String;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if self.parameters.only_fails {
            log::info!("Only fails");
            self.only_fails.poll_next_unpin(cx)
        } else if !self.parameters.include_fails {
            log::info!("not fails");
            self.not_fails.filter(|(_, b)| async move { !b }).map(|(id, _)| id).poll_next_unpin(cx)
        } else {
            log::info!("all chapter");
            self.not_fails.all_chapter.poll_next_unpin(cx)
        }
    }
}
