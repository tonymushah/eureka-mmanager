use std::{task::Poll, pin::Pin};

use tokio_stream::Stream;
use uuid::Uuid;

use super::{OnlyFails, NotIncludeFails, GetAllChapter};

pub struct AsyncGetAllChapter<C, H>
where
    C: Stream<Item = Uuid> + std::marker::Unpin,
    H: Stream<Item = Uuid> + std::marker::Unpin,
{
    pub only_fails: OnlyFails<H>,
    pub parameters: GetAllChapter,
    pub not_fails: NotIncludeFails<C>,
}

impl<C, H> Stream for AsyncGetAllChapter<C, H>
where
    C: Stream<Item = Uuid> + std::marker::Unpin,
    H: Stream<Item = Uuid> + std::marker::Unpin,
{
    type Item = Uuid;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if self.parameters.only_fails {
            //log::info!("Only fails");
            Pin::new(&mut self.only_fails).poll_next(cx)
        } else if !self.parameters.include_fails {
            //log::info!("not fails");
            Pin::new(&mut self.not_fails).poll_next(cx)
        } else {
            //log::info!("all chapter");
            Pin::new(&mut self.not_fails.all_chapter).poll_next(cx)
        }
    }
}

