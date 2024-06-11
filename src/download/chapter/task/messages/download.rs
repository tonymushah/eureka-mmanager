use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering as AtomicOrd},
};

use actix::prelude::*;
use bytes::Bytes;
use mangadex_api_schema_rust::v5::ChapterObject as Object;
use mangadex_api_types_rust::RelationshipType;
use tokio_stream::StreamExt;

use crate::{
    data_push::chapter::{image::ChapterImagePushEntry, ChapterRequiredRelationship},
    download::{
        chapter::task::{ChapterDownloadTask as Task, ChapterDownloadingState as State},
        messages::{state::GetManagerStateMessage, StartDownload, TaskStateMessage},
        state::{
            messages::get::{
                client::GetClientMessage, dir_options::GetDirsOptionsMessage,
                history::GetHistoryMessage,
            },
            DownloadTaskState, TaskState,
        },
        traits::task::Download,
    },
    files_dirs::messages::{
        delete::DeleteChapterImagesMessage,
        pull::chapter::{
            ChapterImageDataPullMessage, ChapterImageDataSaverPullMessage, ChapterImagesPullMessage,
        },
        push::PushDataMessage,
    },
    history::{
        service::messages::{insert::InsertMessage, remove::RemoveMessage},
        HistoryEntry,
    },
    ManagerCoreResult,
};

impl Download for Task {
    fn download(&mut self, ctx: &mut Self::Context) {
        if self.handle(TaskStateMessage, ctx) != TaskState::Loading {
            self.sender
                .send_replace(DownloadTaskState::Loading(State::Preloading));
            let manager = self.manager.clone();
            let mode = self.mode;
            let sender = self.sender.clone();
            let id = self.id;

            let entry = HistoryEntry::new(id, RelationshipType::Chapter);
            if let Some(t) = self.handle.replace(
                ctx.spawn(
                    async move {
                        // Getting manager state data

                        let manager_state = manager.send(GetManagerStateMessage).await?;
                        let client = manager_state.send(GetClientMessage).await?;
                        let dir_options = manager_state.send(GetDirsOptionsMessage).await?;
                        let history = manager_state.send(GetHistoryMessage).await?;
                        // fetching chapter data
                        sender.send_replace(DownloadTaskState::Loading(State::FetchingData));
                        // insert data in history
                        history.send(InsertMessage::new(entry).commit()).await??;
                        let res = client
                            .chapter()
                            .id(id)
                            .get()
                            .includes(ChapterRequiredRelationship::get_includes())
                            .send()
                            .await?;
                        // push chapter data to the dirs_option actor
                        dir_options
                            .send(PushDataMessage::new(res.data.clone()).verify(true))
                            .await??;
                        // Getting fetching AtHome data
                        sender.send_replace(DownloadTaskState::Loading(State::FetchingAtHomeData));
                        let current_images =
                            dir_options.send(ChapterImagesPullMessage(id)).await??;
                        let mut images: HashMap<String, usize> = Default::default();
                        // getting current images size
                        match mode {
                            crate::download::chapter::task::DownloadMode::Normal => {
                                for image in &current_images.data {
                                    if let Ok(b) = dir_options
                                        .send(ChapterImageDataPullMessage(id, image.clone()))
                                        .await?
                                    {
                                        images.insert(image.clone(), b.len());
                                    }
                                }
                            }
                            crate::download::chapter::task::DownloadMode::DataSaver => {
                                for image in &current_images.data_saver {
                                    if let Ok(b) = dir_options
                                        .send(ChapterImageDataSaverPullMessage(id, image.clone()))
                                        .await?
                                    {
                                        images.insert(image.clone(), b.len());
                                    }
                                }
                            }
                        };
                        let is_new = AtomicBool::new(current_images.is_empty());
                        let is_first_loading = AtomicBool::new(true);
                        let stream = client
                            .download()
                            .chapter(id)
                            .report(true)
                            .mode(mode)
                            .build()?
                            .download_stream_with_checker(|at_home, resp| {
                                if !is_new.load(AtomicOrd::Relaxed)
                                    && is_first_loading.load(AtomicOrd::Relaxed)
                                {
                                    match mode {
                                        crate::download::chapter::task::DownloadMode::Normal => {
                                            is_new.swap(
                                                at_home
                                                    .at_home
                                                    .chapter
                                                    .data
                                                    .partial_cmp(&current_images.data)
                                                    .is_some_and(|cm| cm.is_eq()),
                                                AtomicOrd::Relaxed,
                                            );
                                        }
                                        crate::download::chapter::task::DownloadMode::DataSaver => {
                                            is_new.swap(
                                                at_home
                                                    .at_home
                                                    .chapter
                                                    .data_saver
                                                    .partial_cmp(&current_images.data_saver)
                                                    .is_some_and(|cm| cm.is_eq()),
                                                AtomicOrd::Relaxed,
                                            );
                                        }
                                    };
                                    is_first_loading.swap(false, AtomicOrd::Relaxed);
                                }
                                if is_new.load(AtomicOrd::Relaxed) {
                                    false
                                } else {
                                    !resp
                                        .content_length()
                                        .and_then(|cl| {
                                            images
                                                .get(resp.url().path().split('/').next()?)?
                                                .partial_cmp(&cl.try_into().ok()?)
                                        })
                                        .map(|o| o.is_eq())
                                        .unwrap_or_default()
                                }
                            })
                            .await?;
                        // Delete if the chapter data is new
                        if is_new.load(AtomicOrd::Relaxed) {
                            let _ = dir_options
                                .send(
                                    DeleteChapterImagesMessage::new(id, mode).ignore_conflict(true),
                                )
                                .await?;
                        }
                        // Fetches each images and stores it
                        let mut have_error = false;
                        let have_error_ref = &mut have_error;
                        let mut mark_have_error = move || {
                            if !*have_error_ref {
                                *have_error_ref = true;
                            }
                        };
                        let mut stream = Box::pin(stream);
                        while let Some(((filename, res_bytes), index, len)) = stream.next().await {
                            sender.send_replace(DownloadTaskState::Loading(State::FetchingImage {
                                filename: filename.clone(),
                                index,
                                len,
                            }));
                            match res_bytes {
                                Ok(b) => {
                                    if let Err(e) = dir_options
                                        .send(PushDataMessage::new(
                                            ChapterImagePushEntry::new(id, filename.clone(), b)
                                                .mode(mode),
                                        ))
                                        .await?
                                    {
                                        log::error!("[chapter|{id}|{filename}]>write - {e}");
                                    }
                                }
                                Err(e) => {
                                    if let mangadex_api_types_rust::error::Error::SkippedDownload(
                                        _,
                                    ) = &e
                                    {
                                    } else {
                                        mark_have_error();
                                        log::error!("[chapter|{id}|{filename}]>write - {e}");
                                        if let Err(e) = dir_options
                                            .send(PushDataMessage::new(
                                                ChapterImagePushEntry::new(
                                                    id,
                                                    filename.clone(),
                                                    Bytes::new(),
                                                )
                                                .mode(mode),
                                            ))
                                            .await?
                                        {
                                            log::error!("[chapter|{id}|{filename}]>write - {e}");
                                        }
                                    }
                                }
                            }
                        }
                        if !have_error {
                            history.send(RemoveMessage::new(entry).commit()).await??;
                        }
                        Ok(res.data)
                    }
                    .into_actor(self)
                    .map(|res: ManagerCoreResult<Object>, this, _| {
                        let _ = this.sender.send_replace(res.into());
                    }),
                ),
            ) {
                ctx.cancel_future(t);
            }
        }
    }
}

impl Handler<StartDownload> for Task {
    type Result = ();
    fn handle(&mut self, _msg: StartDownload, ctx: &mut Self::Context) -> Self::Result {
        self.download(ctx);
    }
}
