use std::ops::Deref;

use actix::prelude::*;

use crate::download::{
    chapter::task::{ChapterDownloadTask as Task, ChapterDownloadTaskState as State},
    messages::SubcribeMessage,
    traits::task::Subscribe,
};

impl Handler<SubcribeMessage<State>> for Task {
    type Result = <SubcribeMessage<State> as Message>::Result;
    fn handle(&mut self, _msg: SubcribeMessage<State>, _ctx: &mut Self::Context) -> Self::Result {
        self.subscribe(_msg.0)
    }
}

impl Subscribe for Task {
    fn subscribe(
        &mut self,
        subscriber: Recipient<crate::download::messages::TaskSubscriberMessages<Self::State>>,
    ) {
        subscriber.do_send(crate::download::messages::TaskSubscriberMessages::ID(
            self.id,
        ));
        subscriber.do_send(crate::download::messages::TaskSubscriberMessages::State(
            self.state.read().deref().clone(),
        ));
        self.subscribers.push_recipient(subscriber);
    }
}
