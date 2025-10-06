use std::future::Future;

use actix::prelude::*;
use mangadex_api::MangaDexClient;

use crate::{DirsOptions, history::service::HistoryActorService};

use self::{
    chapter::ChapterDownloadManager, cover::CoverDownloadManager, manga::MangaDownloadManager,
    state::DownloadManagerState,
};

pub trait GetManager<T>
where
    T: Actor,
{
    fn get(&self) -> impl Future<Output = Result<Addr<T>, MailboxError>> + Send;
}

pub mod chapter;
pub mod cover;
pub mod manga;
pub mod messages;
pub mod state;
pub mod traits;

#[derive(Debug)]
pub struct DownloadManager {
    state: Addr<DownloadManagerState>,
    manga: Addr<MangaDownloadManager>,
    cover: Addr<CoverDownloadManager>,
    chapter: Addr<ChapterDownloadManager>,
}

impl DownloadManager {
    pub fn new(dir_option: Addr<DirsOptions>, client: MangaDexClient) -> Self {
        let history = HistoryActorService::new(dir_option.clone()).start();
        let state = DownloadManagerState::new(dir_option, client, history).start();
        state.into()
    }
}

impl Actor for DownloadManager {
    type Context = Context<Self>;
}

impl From<Addr<DownloadManagerState>> for DownloadManager {
    fn from(state: Addr<DownloadManagerState>) -> Self {
        Self {
            manga: MangaDownloadManager::new(state.clone()).start(),
            cover: CoverDownloadManager::new(state.clone()).start(),
            chapter: ChapterDownloadManager::new(state.clone()).start(),
            state,
        }
    }
}

macro_rules! impl_dopt_sub_to_manager {
    ($manager:ty) => {
        impl actix::Handler<$crate::files_dirs::messages::subscribe::DirsOptionsSubscribeMessage>
            for $manager
        {
            type Result = ResponseFuture<()>;
            fn handle(
                &mut self,
                msg: $crate::files_dirs::messages::subscribe::DirsOptionsSubscribeMessage,
                _ctx: &mut Self::Context,
            ) -> Self::Result {
                use futures_util::FutureExt;
                let state =
                    <$manager as $crate::download::traits::managers::TaskManager>::state(&self);
                Box::pin(
                    async move {
                        use $crate::prelude::GetManagerStateData;
                        let dir_options = state.get_dir_options().await?;
                        dir_options.send(msg).await?;
                        Ok::<(), $crate::Error>(())
                    }
                    .map(|res| {
                        if let Err(err) = res {
                            log::error!("{err}");
                        }
                    }),
                )
            }
        }
    };
    () => {};
}

impl_dopt_sub_to_manager!(chapter::ChapterDownloadManager);
impl_dopt_sub_to_manager!(cover::CoverDownloadManager);
impl_dopt_sub_to_manager!(manga::MangaDownloadManager);
