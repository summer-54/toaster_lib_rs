pub mod invoker_manager;
pub mod testing_system;

pub mod pb {
    pub mod toaster {
        tonic::include_proto!("toaster");
    }
    pub mod invoker_manager {
        tonic::include_proto!("invoker_manager");
    }
    pub mod testing_system {
        tonic::include_proto!("testing_system");
    }
}

use futures::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::Status;

use crate::{
    judge::{Lang, test},
    prelude::*,
};

use tokio::sync::{Mutex, mpsc::UnboundedSender};
impl TryFrom<pb::toaster::Lang> for Lang {
    type Error = Error;

    fn try_from(proto: pb::toaster::Lang) -> Result<Self, Self::Error> {
        Ok(match proto {
            pb::toaster::Lang::Unspecified => bail!("lang unspecified"),
            pb::toaster::Lang::Gpp => Lang::Gpp,
            pb::toaster::Lang::Python3 => Lang::Python,
        })
    }
}

impl From<Lang> for pb::toaster::Lang {
    fn from(lang: Lang) -> Self {
        match lang {
            Lang::Gpp => pb::toaster::Lang::Gpp,
            Lang::Python => pb::toaster::Lang::Python3,
        }
    }
}

impl TryFrom<pb::toaster::TestVerdict> for test::Verdict {
    type Error = Error;

    fn try_from(proto: pb::toaster::TestVerdict) -> Result<Self, Self::Error> {
        Ok(match proto {
            pb::toaster::TestVerdict::Unspecified => bail!("verdict unspecified"),
            pb::toaster::TestVerdict::Ok => Self::Ok,
            pb::toaster::TestVerdict::Wa => Self::Wa,
            pb::toaster::TestVerdict::Tl => Self::Tl,
            pb::toaster::TestVerdict::Ml => Self::Ml,
            pb::toaster::TestVerdict::Sl => Self::Sl,
            pb::toaster::TestVerdict::Re => Self::Re,
            pb::toaster::TestVerdict::Ce => Self::Ce,
            pb::toaster::TestVerdict::Te => Self::Te,
            pb::toaster::TestVerdict::Pe => Self::Pe,
        })
    }
}

impl From<test::Verdict> for pb::toaster::TestVerdict {
    fn from(verdict: test::Verdict) -> Self {
        match verdict {
            test::Verdict::Ok => pb::toaster::TestVerdict::Ok,
            test::Verdict::Wa => pb::toaster::TestVerdict::Wa,
            test::Verdict::Pe => pb::toaster::TestVerdict::Pe,
            test::Verdict::Ml => pb::toaster::TestVerdict::Ml,
            test::Verdict::Tl => pb::toaster::TestVerdict::Tl,
            test::Verdict::Re => pb::toaster::TestVerdict::Re,
            test::Verdict::Ce => pb::toaster::TestVerdict::Ce,
            test::Verdict::Te => pb::toaster::TestVerdict::Te,
            test::Verdict::Sl => pb::toaster::TestVerdict::Sl,
        }
    }
}

impl TryFrom<pb::toaster::TestResult> for test::Result {
    type Error = Error;

    fn try_from(proto: pb::toaster::TestResult) -> Result<Self, Self::Error> {
        Ok(test::Result {
            verdict: test::Verdict::try_from(proto.verdict())?,
            time: proto.time_sec,
            memory: proto.memory_bytes,
        })
    }
}

impl From<test::Result> for pb::toaster::TestResult {
    fn from(result: test::Result) -> Self {
        Self {
            verdict: pb::toaster::TestVerdict::from(result.verdict).into(),
            time_sec: result.time,
            memory_bytes: result.memory,
        }
    }
}

// Servers =================

pub struct ClientStream<I, O, S: futures::Stream<Item = Result<I, Status>>> {
    receiver: Mutex<S>,
    sender: UnboundedSender<O>,
}

impl<I, O> ClientStream<I, O, tonic::Streaming<I>> {
    pub async fn from_fn<F>(value: F) -> Result<Self>
    where
        F: AsyncFnOnce(
            tonic::Request<tokio_stream::wrappers::UnboundedReceiverStream<O>>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<I>>,
            tonic::Status,
        >,
    {
        let (sender_outgo, receiver_outgo) = tokio::sync::mpsc::unbounded_channel();
        let request = tonic::Request::new(tokio_stream::wrappers::UnboundedReceiverStream::new(
            receiver_outgo,
        ));
        let response = value(request).await.context("gRPC execution")?.into_inner();
        Ok(Self::new(response, sender_outgo))
    }
}

impl<I, O, S: futures::Stream<Item = Result<I, Status>>> ClientStream<I, O, S> {
    pub fn new(receiver: S, sender: UnboundedSender<O>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
            sender,
        }
    }
}

impl<I, O, SI, SO, S, E> super::stream::Stream<I, O> for ClientStream<SI, SO, S>
where
    I: TryFrom<SI, Error = E> + Send,
    O: Into<SO> + Send,
    SO: Send + Sync + 'static,
    S: futures::Stream<Item = Result<SI, Status>> + Unpin + Send,
    E: Into<Error>,
{
    async fn recv(&self) -> anyhow::Result<anyhow::Result<I>> {
        Ok(I::try_from(
            self.receiver
                .lock()
                .await
                .next()
                .await
                .context("receiving message")?
                .context("reading message")?,
        )
        .map_err(Into::into)
        .context("converting message"))
    }

    async fn send(&self, msg: O) -> anyhow::Result<()> {
        self.sender.send(msg.into()).context("sending message")
    }
}

pub struct ServerStream<I, O, S: futures::Stream<Item = Result<I, Status>>> {
    receiver: Mutex<S>,
    sender: UnboundedSender<Result<O, Status>>,
}

impl<I, O, S: futures::Stream<Item = Result<I, Status>>> ServerStream<I, O, S> {
    pub fn new(receiver: S, sender: UnboundedSender<Result<O, Status>>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
            sender,
        }
    }
}

impl<I, O> ServerStream<I, O, tonic::Streaming<I>> {
    pub async fn from_request<F>(
        request: tonic::Request<tonic::Streaming<I>>,
    ) -> (
        Self,
        tonic::Response<UnboundedReceiverStream<Result<O, Status>>>,
    ) {
        let (sender_outgo, receiver_outgo) =
            tokio::sync::mpsc::unbounded_channel::<Result<O, Status>>();

        let receiver = request.into_inner();
        (
            Self::new(receiver, sender_outgo),
            tonic::Response::new(UnboundedReceiverStream::new(receiver_outgo)),
        )
    }
}

impl<I, O, SI, SO, S, E> super::stream::Stream<I, O> for ServerStream<SI, SO, S>
where
    I: TryFrom<SI, Error = E> + Send,
    O: Into<SO> + Send,
    SO: Send + Sync + 'static,
    S: futures::Stream<Item = Result<SI, Status>> + Unpin + Send,
    E: Into<Error>,
{
    async fn recv(&self) -> anyhow::Result<anyhow::Result<I>> {
        Ok(I::try_from(
            self.receiver
                .lock()
                .await
                .next()
                .await
                .context("receiving message")?
                .context("reading message")?,
        )
        .map_err(Into::into)
        .context("converting message"))
    }

    async fn send(&self, msg: O) -> anyhow::Result<()> {
        self.sender.send(Ok(msg.into())).context("sending message")
    }
}
