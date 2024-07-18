/// This allows you sort your [`tokio_stream::Stream`] with some sort of parameter `P`.
///
/// Please note that it returns a [`Vec`].
/// Which means that you need to collect all the stream data and then implement sort code after that.
/// It's recommended to implement [`IntoSorted`] first to avoid having a long code.
pub trait AsyncIntoSorted<P> {
    type Item;
    fn to_sorted(
        self,
        params: P,
    ) -> impl std::future::Future<Output = Vec<<Self as AsyncIntoSorted<P>>::Item>> + Send;
}

/// This allows you sort your [`Vec`] or [`Iterator`] (which I tried but It doesn't work) with some sort of parameter`` P`.
///
/// __But what's the difference between [`[T]::sort_by`](https://doc.rust-lang.org/std/primitive.slice.html#method.sort_by)?__
///
/// Well, if you need to sort something more often with a set of parameters then this is the perfect solution for you.
pub trait IntoSorted<P> {
    type Item;
    fn to_sorted(self, params: P) -> Vec<<Self as IntoSorted<P>>::Item>;
}
