use std::ops::Deref;

use actix::prelude::*;

use crate::download::{
    cover::task::{CoverDownloadTask as Task, CoverDownloadTaskState as State},
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
        subscriber: crate::recipients::MaybeWeakRecipient<
            crate::download::messages::TaskSubscriberMessages<Self::State>,
        >,
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
