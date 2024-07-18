use std::iter::Filter;
#[cfg(feature = "stream")]
use std::{
    pin::Pin,
    task::{ready, Poll},
};

#[cfg(feature = "stream")]
use tokio_stream::Stream;

use super::Validate;

/// A [`Stream`] that filter another [`Stream`] with a Parameter that implement [`Validate`]
/// since filtering don't need to collect the underlying stream.
///
#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
pub struct ParamedFilteredStream<S, P>
where
    S: Stream,
    P: Validate<S::Item>,
{
    stream: Pin<Box<S>>,
    params: P,
}

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
impl<S, P> Unpin for ParamedFilteredStream<S, P>
where
    S: Stream,
    P: Validate<S::Item>,
{
}

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
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

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
impl<S, P> Stream for ParamedFilteredStream<S, P>
where
    S: Stream,
    P: Validate<S::Item>,
{
    type Item = S::Item;
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match ready!(self.as_mut().stream.as_mut().poll_next(cx)) {
            Some(m) => {
                if self.params.is_valid(&m) {
                    Poll::Ready(Some(m))
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            None => Poll::Ready(None),
        }
    }
}

/// Filter an [`Stream`] with a [`Validate`] param.
///
/// Use [`IntoFiltered`] for an synchronous version
#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
pub trait IntoParamedFilteredStream<P>: Stream + Sized
where
    P: Validate<Self::Item>,
{
    fn to_filtered(self, params: P) -> ParamedFilteredStream<Self, P> {
        ParamedFilteredStream::new(self, params)
    }
    fn to_filtered_into<I: Into<P>>(self, params: I) -> ParamedFilteredStream<Self, P> {
        Self::to_filtered(self, params.into())
    }
}

/// Filter an [`Iterator`] with a [`Validate`] param.
///
/// Use [`IntoParamedFilteredStream`] for an asynchronous version
pub trait IntoFiltered<P>: Iterator + Sized
where
    P: Validate<Self::Item>,
{
    fn to_filtered(self, param: P) -> Filter<Self, impl FnMut(&Self::Item) -> bool> {
        self.filter(move |input| param.is_valid(input))
    }
    fn to_filtered_into<I: Into<P>>(
        self,
        params: I,
    ) -> Filter<Self, impl FnMut(&Self::Item) -> bool> {
        let param: P = params.into();
        self.to_filtered(param)
    }
}
