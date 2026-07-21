use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};
pub struct ResourcePool<I> {
    sender: UnboundedSender<I>,
    receiver: Mutex<UnboundedReceiver<I>>,
}

impl<I> FromIterator<I> for ResourcePool<I> {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let this = Self::default();
        for i in iter {
            this.put(i);
        }
        this
    }
}

impl<I> Default for ResourcePool<I> {
    fn default() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
        }
    }
}

impl<I> ResourcePool<I> {
    pub fn put(&self, t: I) {
        self.sender.send(t).unwrap();
    }

    pub async fn take(&self) -> I {
        self.receiver.lock().await.recv().await.unwrap()
    }
}
