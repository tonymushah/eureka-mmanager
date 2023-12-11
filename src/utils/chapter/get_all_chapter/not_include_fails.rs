use std::{task::Poll, pin::Pin};

use tokio_stream::Stream;
use uuid::Uuid;

use crate::settings::file_history::{History, IsIn};

pub struct NotIncludeFails<T>
where
    T: Stream<Item = Uuid> + Unpin,
{
    pub(crate) all_chapter: T,
    history: History,
}

impl<T> NotIncludeFails<T>
where
    T: Stream<Item = Uuid> + Unpin,
{
    pub fn new(all_chapter: T, history: History) -> Self {
        Self {
            all_chapter,
            history,
        }
    }
}

impl<T> Stream for NotIncludeFails<T>
where
    T: Stream<Item = Uuid> + Unpin,
{
    type Item = Uuid;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            if let Poll::Ready(getted) = Pin::new(&mut self.all_chapter).poll_next(cx) {
                if let Some(id) = getted {
                    if self.history.is_in(id).is_none() {
                        return Poll::Ready(Some(id));
                    }
                } else {
                    //log::info!("Exited");
                    return Poll::Ready(None);
                }
            } else {
                //log::info!("Pending 1");
                return Poll::Pending;
            }
        }
    }
}
