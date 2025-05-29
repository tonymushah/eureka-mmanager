use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering as AtomicOrd},
        Arc,
    },
};

use actix::prelude::*;
use bytes::{Buf, Bytes};
use futures_util::FutureExt;
use mangadex_api_schema_rust::v5::ChapterObject as Object;
use mangadex_api_types_rust::RelationshipType;
use tokio_stream::StreamExt;

use crate::{
    data_push::chapter::{image::ChapterImagePushEntry, ChapterRequiredRelationship},
    download::{
        chapter::task::{
            ChapterDownloadTask as Task, ChapterDownloadTaskState, ChapterDownloadingState as State,
        },
        messages::StartDownload,
        state::{messages::get::GetManagerStateData, DownloadTaskState, TaskState},
        traits::task::{Download, State as TaskStateTrait},
    },
    history::{
        history_w_file::traits::{AsyncAutoCommitRollbackInsert, AsyncAutoCommitRollbackRemove},
        HistoryEntry,
    },
    prelude::{ChapterDataPullAsyncTrait, DeleteDataAsyncTrait, PushActorAddr},
    ManagerCoreResult,
};

impl Task {
    fn preloading(&self) {
        *self.state.write() = DownloadTaskState::Loading(State::Preloading);
        self.sync_state_subscribers();
    }
    fn send_to_subscrbers(&self) -> Arc<dyn Fn(ChapterDownloadTaskState) + Send + Sync + 'static> {
        let state = self.state.clone();
        let subs = self.subscribers.clone();
        Arc::new({
            move |state_to_send: ChapterDownloadTaskState| {
                *state.write() = state_to_send.clone();
                subs.do_send(crate::download::messages::TaskSubscriberMessages::State(
                    state_to_send,
                ));
            }
        })
    }
}

impl Download for Task {
    fn download(&mut self, ctx: &mut Self::Context) {
        if self.state() != TaskState::Loading {
            self.preloading();
            let manager = self.manager.clone();
            let mode = self.mode;
            let id = self.id;

            let entry = HistoryEntry::new(id, RelationshipType::Chapter);
            let send_to_subscrbers = self.send_to_subscrbers();
            let send_to_subs_map = send_to_subscrbers.clone();
            if let Some(t) = self.handle.replace(
                ctx.spawn(
                    async move {
                        // Getting manager state data

                        let client = manager.get_client().await?;
                        let mut history = manager.get_history().await?;
                        // fetching chapter data
                        send_to_subscrbers(DownloadTaskState::Loading(State::FetchingData));
                        // insert data in history
                        history.insert_and_commit(entry).await?;
                        let res = client
                            .chapter()
                            .id(id)
                            .get()
                            .includes(ChapterRequiredRelationship::get_includes())
                            .send()
                            .await?;
                        // push chapter data to the dirs_option actor
                        manager.verify_and_push(res.data.clone()).await?;
                        // Getting fetching AtHome data
                        send_to_subscrbers(DownloadTaskState::Loading(State::FetchingAtHomeData));
                        let current_images = manager.get_chapter_images(id).await?;
                        let mut images: HashMap<String, usize> = Default::default();
                        // getting current images size
                        match mode {
                            crate::download::chapter::task::DownloadMode::Normal => {
                                for image in &current_images.data {
                                    if let Ok(b) =
                                        manager.get_chapter_image(id, image.clone()).await
                                    {
                                        if let Ok(len) = b.metadata().map(|met| met.len() as usize)
                                        {
                                            images.insert(image.clone(), len);
                                        }
                                    }
                                }
                            }
                            crate::download::chapter::task::DownloadMode::DataSaver => {
                                for image in &current_images.data_saver {
                                    if let Ok(b) = manager
                                        .get_chapter_image_data_saver(id, image.clone())
                                        .await
                                    {
                                        if let Ok(len) = b.metadata().map(|met| met.len() as usize)
                                        {
                                            images.insert(image.clone(), len);
                                        }
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
                                                .partial_cmp(&TryInto::<usize>::try_into(cl).ok()?)
                                        })
                                        .map(|o| o.is_eq())
                                        .unwrap_or_default()
                                }
                            })
                            .await?;
                        // Delete if the chapter data is new
                        if is_new.load(AtomicOrd::Relaxed) {
                            manager
                                .delete_chapter_images_ignore_conflict(id, mode)
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
                            send_to_subscrbers(DownloadTaskState::Loading(State::FetchingImage {
                                filename: filename.clone(),
                                index,
                                len,
                            }));
                            match res_bytes {
                                Ok(b) => {
                                    if let Err(e) = manager
                                        .push(
                                            ChapterImagePushEntry::new(
                                                id,
                                                filename.clone(),
                                                b.reader(),
                                            )
                                            .mode(mode),
                                        )
                                        .await
                                    {
                                        mark_have_error();
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
                                        if let Err(e) = manager
                                            .push(
                                                ChapterImagePushEntry::new(
                                                    id,
                                                    filename.clone(),
                                                    Bytes::new().reader(),
                                                )
                                                .mode(mode),
                                            )
                                            .await
                                        {
                                            mark_have_error();
                                            log::error!("[chapter|{id}|{filename}]>write - {e}");
                                        }
                                    }
                                }
                            }
                        }
                        if !have_error {
                            history.remove_and_commit(entry).await?;
                        }
                        Ok(res.data)
                    }
                    .map(move |res: ManagerCoreResult<Object>| {
                        send_to_subs_map(res.into());
                    })
                    .into_actor(self),
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
