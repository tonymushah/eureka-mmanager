pub use crate::r#core::ManagerCoreResult;

pub mod download;
pub mod files_dirs;
pub mod history;
pub mod recipients;

type MailBoxResult<T, E = actix::MailboxError> = Result<T, E>;

mod r#core;

pub(crate) type ArcRwLock<T> = std::sync::Arc<parking_lot::RwLock<T>>;

pub use crate::r#core::{Error, ErrorType, OwnedError};

pub use download::DownloadManager;
pub use files_dirs::DirsOptions;

pub(crate) mod data_pulls {
    pub use api_core::data_pulls::*;
}

pub(crate) mod data_push {
    pub use api_core::data_push::*;
}

/// The `mangadex-desktop-api2` prelude module
pub mod prelude {
    pub use super::{
        Error, ManagerCoreResult, OwnedError,
        download::{
            DownloadManager, GetManager,
            chapter::{ChapterDownloadManager, task::ChapterDownloadTask},
            cover::{CoverDownloadManager, task::CoverDownloadTask},
            manga::{MangaDownloadManager, task::MangaDownloadTask},
            messages::{
                chapter::GetChapterDownloadManager, cover::GetCoverDownloadManager,
                manga::GetMangaDownloadManager,
            },
            state::{
                DownloadManagerState,
                messages::{get::GetManagerStateData, update::UpdateManagerStateData},
            },
            traits::{
                managers::TaskManagerAddr,
                task::{
                    AsyncCanBeWaited, AsyncCancelable, AsyncDownload, AsyncState, AsyncSubscribe,
                },
            },
        },
        files_dirs::{
            DirsOptions,
            messages::{
                delete::DeleteDataAsyncTrait,
                join::JoinPathAsyncTraits,
                modify::ModifyDirOptionAsyncTrait,
                pull::{
                    chapter::ChapterDataPullAsyncTrait, cover::CoverDataPullAsyncTrait,
                    manga::MangaDataPullAsyncTrait,
                },
                push::PushActorAddr,
            },
        },
        history::{
            AsyncInsert, AsyncIsIn, AsyncRemove, HistoryEntry, history_w_file::traits::*,
            service::HistoryActorService,
        },
    };
    pub use api_core::{
        data_pulls::{
            AsyncIntoSorted, AsyncPaginate, AsyncRand, IntoFiltered, IntoParamedFilteredStream,
            IntoSorted, Paginate, Rand,
            chapter::ChapterListDataPullFilterParams,
            cover::CoverListDataPullFilterParams,
            manga::{
                MangaListDataPullFilterParams,
                aggregate::{AsyncIntoMangaAggreagate, IntoMangaAggreagate},
            },
        },
        data_push::{Push, chapter::image::ChapterImagePushEntry},
        file_dirs::DirsOptions as DirsOptionsCore,
    };
}
