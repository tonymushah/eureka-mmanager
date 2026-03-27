use std::time::Duration;

use actix::prelude::*;

use crate::{
    download::{
        messages::{StartDownload, TaskSubscriberMessages, WaitForFinishedMessage},
        state::{DownloadTaskState, make_wait_for_finish_couple},
    },
    recipients::Recipients,
};

#[derive(Debug, Default)]
pub struct TestActor {
    task_a: Option<SpawnHandle>,
    listeners: Recipients<TaskSubscriberMessages<DownloadTaskState<(), ()>>>,
}

impl Actor for TestActor {
    type Context = Context<Self>;
}

impl Handler<StartDownload> for TestActor {
    type Result = ();
    fn handle(&mut self, _: StartDownload, ctx: &mut Self::Context) -> Self::Result {
        if self.task_a.is_none() {
            let handle = {
                let listeners = self.listeners.clone();

                async move {
                    listeners.do_send(TaskSubscriberMessages::State(
                        DownloadTaskState::Loading(()),
                    ));
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
                .into_actor(self)
                .map(|_, this, _| {
                    this.listeners
                        .do_send(TaskSubscriberMessages::State(DownloadTaskState::Done(())));
                    this.task_a.take();
                })
            };
            self.task_a = Some(ctx.spawn(handle));
        }
    }
}

impl Handler<WaitForFinishedMessage<(), ()>> for TestActor {
    type Result = <WaitForFinishedMessage<(), ()> as Message>::Result;
    fn handle(&mut self, _: WaitForFinishedMessage<(), ()>, _: &mut Self::Context) -> Self::Result {
        let (rec, wait) = make_wait_for_finish_couple::<(), ()>();
        self.listeners.push_recipient(rec.into());
        wait
    }
}

#[actix::test]
async fn run_wait_test() -> anyhow::Result<()> {
    let act = TestActor::default().start();
    act.send(StartDownload).await?;
    let wait = act.send(WaitForFinishedMessage::<(), ()>::new()).await?;
    tokio::spawn(tokio::time::timeout(Duration::from_secs(4), wait)).await???;

    Ok(())
}
