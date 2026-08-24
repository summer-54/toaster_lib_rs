mod auth;
mod judge;
mod master;

pub mod pb {
    tonic::include_proto!("invoker_manager");
}

use futures::StreamExt;
pub use pb::invoker_manager_service_client::InvokerManagerServiceClient as Client;
pub use pb::invoker_manager_service_server::{
    InvokerManagerService as Service, InvokerManagerServiceServer as Server,
};

use crate::prelude::*;

use tokio::sync::{Mutex, mpsc::UnboundedSender};

pub struct Stream<I, O, S: futures::Stream<Item = I>> {
    receiver: Mutex<S>,
    sender: UnboundedSender<O>,
}

impl<I, O, S: futures::Stream<Item = I>> Stream<I, O, S> {
    pub fn new(receiver: S, sender: UnboundedSender<O>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
            sender,
        }
    }
}

impl<I, O, SI, SO, S> super::stream::Stream<I, O> for Stream<SI, SO, S>
where
    SI: TryInto<I, Error = Error> + Send,
    O: Into<SO> + Send,
    SO: Send + Sync + 'static,
    S: futures::Stream<Item = SI> + Unpin + Send,
{
    fn recv(&self) -> impl Future<Output = anyhow::Result<anyhow::Result<I>>> + Send {
        async {
            self.receiver
                .lock()
                .await
                .next()
                .await
                .context("receiving message")
                .map(|msg| msg.try_into().context("converting message"))
        }
    }

    fn send(&self, msg: O) -> impl Future<Output = anyhow::Result<()>> + Send {
        async { self.sender.send(msg.into()).context("sending message") }
    }
}
