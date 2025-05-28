pub use crate::r#core::ManagerCoreResult;

pub mod download;
pub mod files_dirs;
pub mod history;
pub(crate) mod recipients;

type MailBoxResult<T, E = actix::MailboxError> = Result<T, E>;

mod r#core;

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
        download::{
            chapter::{task::ChapterDownloadTask, ChapterDownloadManager},
            cover::{task::CoverDownloadTask, CoverDownloadManager},
            manga::{task::MangaDownloadTask, MangaDownloadManager},
            messages::{
                chapter::GetChapterDownloadManager, cover::GetCoverDownloadManager,
                manga::GetMangaDownloadManager,
            },
            state::{
                messages::{get::GetManagerStateData, update::UpdateManagerStateData},
                DownloadManagerState,
            },
            traits::{
                managers::TaskManagerAddr,
                task::{
                    AsyncCanBeWaited, AsyncCancelable, AsyncDownload, AsyncState, AsyncSubscribe,
                },
            },
            DownloadManager, GetManager,
        },
        files_dirs::{
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
            DirsOptions,
        },
        history::{
            history_w_file::traits::*, service::HistoryActorService, AsyncInsert, AsyncIsIn,
            AsyncRemove, HistoryEntry,
        },
        Error, ManagerCoreResult, OwnedError,
    };
    pub use api_core::{
        data_pulls::{
            chapter::ChapterListDataPullFilterParams,
            cover::CoverListDataPullFilterParams,
            manga::{
                aggregate::{AsyncIntoMangaAggreagate, IntoMangaAggreagate},
                MangaListDataPullFilterParams,
            },
            AsyncIntoSorted, AsyncPaginate, AsyncRand, IntoFiltered, IntoParamedFilteredStream,
            IntoSorted, Paginate, Rand,
        },
        data_push::{chapter::image::ChapterImagePushEntry, Push},
        file_dirs::DirsOptions as DirsOptionsCore,
    };
}
