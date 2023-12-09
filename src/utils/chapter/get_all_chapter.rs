use std::pin::Pin;
use std::task::Poll;

use crate::settings::file_history::{History, IsIn};

use super::GetAllChapter;
use tokio_stream::Stream;

pub struct OnlyFails<T>
where
    T: Stream<Item = String> + Unpin,
{
    inner: T,
}

impl<T> OnlyFails<T>
where
    T: Stream<Item = String> + Unpin,
{
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> Stream for OnlyFails<T>
where
    T: Stream<Item = String> + Unpin,
{
    type Item = String;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

pub struct NotIncludeFails<T>
where
    T: Stream<Item = String> + Unpin,
{
    all_chapter: T,
    history: History,
}

impl<T> NotIncludeFails<T>
where
    T: Stream<Item = String> + Unpin,
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
    T: Stream<Item = String> + Unpin,
{
    type Item = String;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            if let Poll::Ready(getted) = Pin::new(&mut self.all_chapter).poll_next(cx) {
                if let Some(id) = getted {
                    if let Ok(uuid) = uuid::Uuid::parse_str(id.clone().as_str()) {
                        if self.history.is_in(uuid).is_none() {
                            return Poll::Ready(Some(id.clone()));
                        }
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

pub struct AsyncGetAllChapter<C, H>
where
    C: Stream<Item = String> + std::marker::Unpin,
    H: Stream<Item = String> + std::marker::Unpin,
{
    pub only_fails: OnlyFails<H>,
    pub parameters: GetAllChapter,
    pub not_fails: NotIncludeFails<C>,
}

impl<C, H> Stream for AsyncGetAllChapter<C, H>
where
    C: Stream<Item = String> + std::marker::Unpin,
    H: Stream<Item = String> + std::marker::Unpin,
{
    type Item = String;

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

#[cfg(test)]
mod test {
    use tokio_stream::StreamExt;

    use crate::{
        server::{traits::AccessHistory, AppState},
        utils::chapter::get_all_chapter::NotIncludeFails,
    };

    #[tokio::test]
    async fn test_not_fails() {
        let app_state = AppState::init().await.unwrap();
        let binding = app_state.chapter_utils();
        let all_chapter = Box::pin(binding.get_all_chapter_without_history().unwrap());
        let history = app_state
            .get_history_w_file_by_rel_or_init(mangadex_api_types_rust::RelationshipType::Chapter)
            .await
            .unwrap();
        let stream =
            NotIncludeFails::new(all_chapter, history.owned_read_history().unwrap().clone());
        let mut stream = Box::pin(stream);
        while let Some(id) = stream.next().await {
            println!("{id}")
        }
    }
}
