pub trait IntoSorted<P> {
    type Item;
    fn to_sorted(
        self,
        params: P,
    ) -> impl std::future::Future<Output = Vec<<Self as IntoSorted<P>>::Item>> + Send;
}
