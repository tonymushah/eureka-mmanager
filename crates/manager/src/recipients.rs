use actix::{dev::RecipientRequest, Message, Recipient, WeakRecipient};

use crate::ArcRwLock;

#[derive(Clone, Debug)]
pub enum MaybeWeakRecipient<M>
where
    M: Message + Send,
    M::Result: Send,
{
    Strong(Recipient<M>),
    Weak(WeakRecipient<M>),
}

impl<M> MaybeWeakRecipient<M>
where
    M: Message + Send,
    M::Result: Send,
{
    pub fn as_weak(&self) -> WeakRecipient<M> {
        match self {
            MaybeWeakRecipient::Strong(recipient) => recipient.downgrade(),
            MaybeWeakRecipient::Weak(weak_recipient) => weak_recipient.clone(),
        }
    }
    pub fn as_strong(&self) -> Option<Recipient<M>> {
        match self {
            MaybeWeakRecipient::Strong(recipient) => Some(recipient.clone()),
            MaybeWeakRecipient::Weak(weak_recipient) => weak_recipient.upgrade(),
        }
    }
    pub fn make_weak(&mut self) {
        *self = Self::Weak(self.as_weak());
    }
    pub fn make_strong(&mut self) {
        if let Some(strong) = self.as_strong() {
            *self = Self::Strong(strong);
        }
    }
    pub fn connected(&self) -> bool {
        self.as_strong().is_some_and(|d| d.connected())
    }
    /// `true` indicate that the [`Recipient::do_send`] was called, `false` otherwise.
    pub fn do_send(&self, message: M) -> bool {
        if let Some(recept) = self.as_strong() {
            recept.do_send(message);
            true
        } else {
            false
        }
    }
    pub fn send(&self, message: M) -> Option<RecipientRequest<M>> {
        self.as_strong().map(|d| d.send(message))
    }
}

impl<M> From<Recipient<M>> for MaybeWeakRecipient<M>
where
    M: Message + Send,
    M::Result: Send,
{
    fn from(value: Recipient<M>) -> Self {
        Self::Strong(value)
    }
}

impl<M> From<WeakRecipient<M>> for MaybeWeakRecipient<M>
where
    M: Message + Send,
    M::Result: Send,
{
    fn from(value: WeakRecipient<M>) -> Self {
        Self::Weak(value)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Recipients<M>(ArcRwLock<Vec<MaybeWeakRecipient<M>>>)
where
    M: Message + Send,
    M::Result: Send;

impl<M> Default for Recipients<M>
where
    M: Message + Send,
    M::Result: Send,
{
    fn default() -> Self {
        Self(ArcRwLock::default())
    }
}

impl<M> Recipients<M>
where
    M: Message + Send,
    M::Result: Send,
{
    fn clean_up(&self) {
        self.0.write().retain(|e| e.connected());
    }
    pub fn push_recipient(&self, recipient: MaybeWeakRecipient<M>) {
        {
            let mut write = self.0.write();
            write.push(recipient);
        }
        self.clean_up();
    }
    pub fn has_connection(&self) -> bool {
        self.0.read().iter().any(|r| r.connected())
    }
}

impl<M> Recipients<M>
where
    M: Message + Clone + Send,
    M::Result: Send,
{
    pub fn do_send(&self, message: M) {
        self.0
            .write()
            .retain(|recipient| recipient.do_send(message.clone()));
    }
}
