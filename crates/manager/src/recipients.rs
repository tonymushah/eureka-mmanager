use actix::{Message, Recipient};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Recipients<M>(Arc<RwLock<Vec<Recipient<M>>>>)
where
    M: Message + Send,
    M::Result: Send;

impl<M> Recipients<M>
where
    M: Message + Send,
    M::Result: Send,
{
    fn clean_up(&self) {
        self.0.write().retain(|e| e.connected());
    }
    pub fn push_recipient(&self, recipient: Recipient<M>) {
        {
            let mut write = self.0.write();
            if !write.contains(&recipient) {
                write.push(recipient);
            }
        }
        self.clean_up();
    }
}

impl<M> Recipients<M>
where
    M: Message + Clone + Send,
    M::Result: Send,
{
    pub fn do_send(&self, message: M) {
        self.clean_up();
        self.0.read().iter().for_each(|recipient| {
            recipient.do_send(message.clone());
        });
    }
}
