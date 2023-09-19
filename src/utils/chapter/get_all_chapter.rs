use std::task::Poll;

use crate::settings::file_history::{History, IsIn};

use super::GetAllChapter;
use futures::StreamExt;
use tokio::sync::OwnedRwLockReadGuard;
use tokio_stream::Stream;

pub struct AsyncGetAllChapter<C, H>
where
    C: Stream<Item = String> + std::marker::Unpin,
    H: Stream<Item = uuid::Uuid> + std::marker::Unpin,
{
    parameters: GetAllChapter,
    history: OwnedRwLockReadGuard<History>,
    all_chapter: C,
    all_history_entry: H,
}

impl<C, H> AsyncGetAllChapter<C, H>
where
    C: Stream<Item = String> + std::marker::Unpin,
    H: Stream<Item = uuid::Uuid> + std::marker::Unpin,
{
    pub fn new(
        parameters: GetAllChapter,
        history: OwnedRwLockReadGuard<History>,
        all_chapter: C,
        all_history_entry: H,
    ) -> Self {
        Self {
            parameters,
            history,
            all_chapter,
            all_history_entry,
        }
    }
}

impl<C, H> tokio_stream::Stream for AsyncGetAllChapter<C, H>
where
    C: Stream<Item = String> + std::marker::Unpin,
    H: Stream<Item = uuid::Uuid> + std::marker::Unpin,
{
    type Item = String;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if !self.parameters.only_fails {
            if let Poll::Ready(data_) = self.all_chapter.poll_next_unpin(cx) {
                if let Some(data) = data_ {
                    if !self.parameters.include_fails {
                        let id = match uuid::Uuid::parse_str(data.as_str()) {
                            Ok(o) => o,
                            Err(_) => uuid::Uuid::NAMESPACE_DNS,
                        };
                        if self.history.is_in(id).is_none() {
                            Poll::Ready(Some(id.to_string()))
                        } else {
                            Poll::Pending
                        }
                    } else {
                        Poll::Ready(Some(data.to_string()))
                    }
                } else {
                    Poll::Ready(None)
                }
            } else {
                Poll::Pending
            }
        } else if let Poll::Ready(data) = self.all_history_entry.poll_next_unpin(cx) {
            if let Some(id) = data {
                Poll::Ready(Some(id.to_string()))
            } else {
                Poll::Ready(None)
            }
        } else {
            Poll::Pending
        }
    }
}
