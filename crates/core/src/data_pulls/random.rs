use rand::random;
#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
use tokio_stream::{Stream, StreamExt};

/// Get a random value asynchronously
pub trait AsyncRand {
    type Output;
    fn random(self) -> impl std::future::Future<Output = Option<Self::Output>> + Send;
}

// Get a random value synchronously
pub trait Rand {
    type Output;
    fn random(self) -> Option<Self::Output>;
}

impl<T: Clone> Rand for Vec<T> {
    type Output = T;
    fn random(self) -> Option<Self::Output> {
        let random_ = random::<u64>() as usize;
        self.get(random_ % self.len()).cloned()
    }
}

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
impl<S, T> AsyncRand for S
where
    S: Stream<Item = T> + Send,
    T: Clone + Send,
{
    type Output = S::Item;
    async fn random(self) -> Option<Self::Output> {
        let stream = Box::pin(self);
        let data = stream.collect::<Vec<_>>().await;
        data.random()
    }
}
