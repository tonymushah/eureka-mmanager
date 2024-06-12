use actix::prelude::*;
pub mod managers;
pub mod task;

type MailBoxResult<T, E = MailboxError> = Result<T, E>;
