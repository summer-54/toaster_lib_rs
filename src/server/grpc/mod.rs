mod auth;
mod judge;
mod master;
pub mod metadata;

pub mod pb {
    tonic::include_proto!("invoker_manager");
}

use futures::StreamExt;
pub use pb::invoker_manager_service_client::InvokerManagerServiceClient as Client;
pub use pb::invoker_manager_service_server::{
    InvokerManagerService as Service, InvokerManagerServiceServer as Server,
};
use tonic::Status;

use crate::prelude::*;

use tokio::sync::{Mutex, mpsc::UnboundedSender};

pub struct ClientStream<I, O, S: futures::Stream<Item = Result<I, Status>>> {
    receiver: Mutex<S>,
    sender: UnboundedSender<O>,
}

impl<I, O, S: futures::Stream<Item = Result<I, Status>>> ClientStream<I, O, S> {
    pub fn new(receiver: S, sender: UnboundedSender<O>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
            sender,
        }
    }
}

impl<I, O, SI, SO, S> super::stream::Stream<I, O> for ClientStream<SI, SO, S>
where
    SI: TryInto<I, Error = Error> + Send,
    O: Into<SO> + Send,
    SO: Send + Sync + 'static,
    S: futures::Stream<Item = Result<SI, Status>> + Unpin + Send,
{
    async fn recv(&self) -> anyhow::Result<anyhow::Result<I>> {
        Ok(self
            .receiver
            .lock()
            .await
            .next()
            .await
            .context("receiving message")?
            .context("reading message")?
            .try_into()
            .context("converting message"))
    }

    async fn send(&self, msg: O) -> anyhow::Result<()> {
        self.sender.send(msg.into()).context("sending message")
    }
}

pub struct ServerStream<I, O, S: futures::Stream<Item = I>> {
    receiver: Mutex<S>,
    sender: UnboundedSender<Result<O, Status>>,
}

impl<I, O, S: futures::Stream<Item = I>> ServerStream<I, O, S> {
    pub fn new(receiver: S, sender: UnboundedSender<Result<O, Status>>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
            sender,
        }
    }
}

impl<I, O, SI, SO, S> super::stream::Stream<I, O> for ServerStream<SI, SO, S>
where
    SI: TryInto<I, Error = Error> + Send,
    O: Into<SO> + Send,
    SO: Send + Sync + 'static,
    S: futures::Stream<Item = SI> + Unpin + Send,
{
    async fn recv(&self) -> anyhow::Result<anyhow::Result<I>> {
        Ok({
            Ok(self
                .receiver
                .lock()
                .await
                .next()
                .await
                .context("receiving message")?
                .try_into()
                .context("converting message")?)
        })
    }

    async fn send(&self, msg: O) -> anyhow::Result<()> {
        self.sender.send(Ok(msg.into())).context("sending message")
    }
}
