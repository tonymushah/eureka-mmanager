use std::{task::Poll, pin::Pin};

use tokio_stream::Stream;
use uuid::Uuid;

pub struct OnlyFails<T>
where
    T: Stream<Item = Uuid> + Unpin,
{
    inner: T,
}

impl<T> OnlyFails<T>
where
    T: Stream<Item = Uuid> + Unpin,
{
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> Stream for OnlyFails<T>
where
    T: Stream<Item = Uuid> + Unpin,
{
    type Item = Uuid;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}
