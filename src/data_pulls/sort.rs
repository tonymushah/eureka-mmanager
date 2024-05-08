pub trait AsyncIntoSorted<P> {
    type Item;
    fn to_sorted(
        self,
        params: P,
    ) -> impl std::future::Future<Output = Vec<<Self as AsyncIntoSorted<P>>::Item>> + Send;
}

pub trait IntoSorted<P> {
    type Item;
    fn to_sorted(self, params: P) -> Vec<<Self as IntoSorted<P>>::Item>;
}
