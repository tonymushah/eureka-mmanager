use actix::{Handler, Message};

use crate::{
    DirsOptions, files_dirs::events::FilesDirSubscriberMessage, recipients::MaybeWeakRecipient,
};

#[derive(Debug, Clone, Message)]
#[rtype("()")]
pub struct DirsOptionsSubscribeMessage(pub MaybeWeakRecipient<FilesDirSubscriberMessage>);

impl Handler<DirsOptionsSubscribeMessage> for DirsOptions {
    type Result = ();
    fn handle(
        &mut self,
        msg: DirsOptionsSubscribeMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.subscribers.push_recipient(msg.0);
    }
}
