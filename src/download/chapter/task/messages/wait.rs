use actix::prelude::*;
use mangadex_api_schema_rust::v5::ChapterObject as Object;

use crate::download::{
    chapter::task::{ChapterDownloadTask as Task, ChapterDownloadingState as State},
    messages::WaitForFinishedMessage,
    state::WaitForFinished,
};

impl Handler<WaitForFinishedMessage<Object, State>> for Task {
    type Result = <WaitForFinishedMessage<Object, State> as Message>::Result;
    fn handle(
        &mut self,
        _msg: WaitForFinishedMessage<Object, State>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        if !self.have_been_read {
            self.have_been_read = true;
        }
        WaitForFinished::new(self.sender.subscribe())
    }
}
