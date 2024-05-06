use std::{
    pin::Pin,
    task::{ready, Poll},
};

use tokio_stream::Stream;

use super::Validate;

pub struct ParamedFilteredStream<S, P>
where
    S: Stream,
    P: Validate<S::Item>,
{
    stream: Pin<Box<S>>,
    params: P,
}

impl<S, P> Unpin for ParamedFilteredStream<S, P>
where
    S: Stream,
    P: Validate<S::Item>,
{
}

impl<S, P> ParamedFilteredStream<S, P>
where
    S: Stream,
    P: Validate<S::Item>,
{
    pub fn new(stream: S, params: P) -> Self {
        Self {
            stream: Box::pin(stream),
            params,
        }
    }
}

impl<S, P> Stream for ParamedFilteredStream<S, P>
where
    S: Stream,
    P: Validate<S::Item>,
{
    type Item = S::Item;
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            match ready!(self.as_mut().stream.as_mut().poll_next(cx)) {
                Some(m) => {
                    if self.params.is_valid(&m) {
                        return Poll::Ready(Some(m));
                    }
                }
                None => return Poll::Ready(None),
            }
        }
    }
}

pub trait IntoParamedFilteredStream: Stream + Sized {
    type Params: Validate<Self::Item>;
    fn to_filtered(self, params: Self::Params) -> ParamedFilteredStream<Self, Self::Params> {
        ParamedFilteredStream::new(self, params)
    }
}
