mod auth;
mod judge;
mod master;

pub mod pb {
    tonic::include_proto!("invoker_manager");
}

pub use pb::invoker_manager_service_client::InvokerManagerServiceClient as Client;
pub use pb::invoker_manager_service_server::{
    InvokerManagerService as Service, InvokerManagerServiceServer as Server,
};

use crate::prelude::*;

use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

pub struct Stream<I, O> {
    receiver: Mutex<UnboundedReceiver<I>>,
    sender: UnboundedSender<O>,
}

impl<I, O> Stream<I, O> {
    pub fn new(receiver: UnboundedReceiver<I>, sender: UnboundedSender<O>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
            sender,
        }
    }
}

impl<I, O, SI, SO> super::stream::Stream<I, O> for Stream<SI, SO>
where
    SI: TryInto<I, Error = Error> + Send,
    O: Into<SO> + Send,
    SO: Send + Sync + 'static,
{
    fn recv(&self) -> impl Future<Output = anyhow::Result<anyhow::Result<I>>> + Send {
        async {
            self.receiver
                .lock()
                .await
                .recv()
                .await
                .context("receiving message")
                .map(|msg| msg.try_into().context("converting message"))
        }
    }

    fn send(&self, msg: O) -> impl Future<Output = anyhow::Result<()>> + Send {
        async { self.sender.send(msg.into()).context("sending message") }
    }
}
