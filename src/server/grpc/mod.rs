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

use testing_system::submission_id;

use crate::judge::{Lang, test};
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConvertError {
    #[error("provided enum was unspecified value")]
    EnumUnspecified,
    #[error("provided struct's field undefined")]
    FieldUndefined,
    #[error("provided struct's field '{field}' convert error: {error}")]
    FieldConvertError {
        field: &'static str,
        error: Box<ConvertError>,
    },
    #[error("parse submission id error {0}")]
    ParseSubmissionId(#[from] submission_id::ParseError),
    #[error("{0}")]
    UnknowEnumValue(#[from] prost::UnknownEnumValue),
    #[error("{0}")]
    Other(Box<str>),
}

pub trait FieldContext {
    fn at_field(self, field: &'static str) -> Self;
}

impl FieldContext for ConvertError {
    fn at_field(self, field: &'static str) -> Self {
        Self::FieldConvertError {
            field,
            error: Box::new(self),
        }
    }
}

impl<O> FieldContext for Result<O, ConvertError> {
    fn at_field(self, field: &'static str) -> Self {
        self.map_err(|e| e.at_field(field))
    }
}

fn unwrap_field<I>(field_name: &'static str, field: Option<I>) -> Result<I, ConvertError> {
    field
        .ok_or(ConvertError::FieldUndefined)
        .map_err(|e| e.at_field(field_name))
}

fn convert_field<I, T, E>(field_name: &'static str, field: I) -> Result<T, ConvertError>
where
    I: TryInto<T, Error = E>,
    E: Into<ConvertError>,
{
    field.try_into().map_err(|e| e.into().at_field(field_name))
}

fn convert_unwrap_field<I, T, E>(
    field_name: &'static str,
    field: Option<I>,
) -> Result<T, ConvertError>
where
    I: TryInto<T, Error = E>,
    E: Into<ConvertError>,
{
    convert_field(field_name, unwrap_field(field_name, field)?)
}

use tokio::sync::{
    Mutex,
    mpsc::{UnboundedSender, error::SendError},
};
impl TryFrom<pb::toaster::Lang> for Lang {
    type Error = ConvertError;

    fn try_from(proto: pb::toaster::Lang) -> Result<Self, Self::Error> {
        Ok(match proto {
            pb::toaster::Lang::Unspecified => return Err(ConvertError::EnumUnspecified),
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
    type Error = ConvertError;

    fn try_from(proto: pb::toaster::TestVerdict) -> Result<Self, Self::Error> {
        Ok(match proto {
            pb::toaster::TestVerdict::Unspecified => return Err(ConvertError::EnumUnspecified),
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
            test::Verdict::Ok => Self::Ok,
            test::Verdict::Wa => Self::Wa,
            test::Verdict::Pe => Self::Pe,
            test::Verdict::Ml => Self::Ml,
            test::Verdict::Tl => Self::Tl,
            test::Verdict::Re => Self::Re,
            test::Verdict::Ce => Self::Ce,
            test::Verdict::Te => Self::Te,
            test::Verdict::Sl => Self::Sl,
        }
    }
}

impl TryFrom<pb::toaster::TestResult> for test::Result {
    type Error = ConvertError;

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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StreamBrokenError<T> {
    #[error("stream was closed")]
    Recv,
    #[error("stream send error: {0}")]
    Send(#[from] SendError<T>),
}
#[derive(Debug, Clone, thiserror::Error)]
pub enum StreamAliveError<P: std::error::Error> {
    #[error("stream parsing message error")]
    Parsing(P),
    #[error("grpc error status {0}")]
    Grpc(#[from] Status),
}
pub struct ClientStream<I, O, S: futures::Stream<Item = Result<I, Status>>> {
    receiver: Mutex<S>,
    sender: UnboundedSender<O>,
}

impl<I, O> ClientStream<I, O, tonic::Streaming<I>> {
    pub async fn from_fn<F>(value: F) -> Result<Self, Status>
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
        let response = value(request).await?.into_inner();
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

impl<I, O, SI, SO, S, AE> super::stream::Stream<I, O> for ClientStream<SI, SO, S>
where
    I: TryFrom<SI, Error = AE> + Send,
    O: Into<SO> + Send,
    SO: Send + Sync + 'static + std::fmt::Debug,
    S: futures::Stream<Item = Result<SI, Status>> + Unpin + Send,
    AE: std::error::Error,
{
    async fn recv(&self) -> Result<I, super::stream::Error<Self::AliveError, Self::BrokenError>> {
        I::try_from(
            self.receiver
                .lock()
                .await
                .next()
                .await
                .ok_or(super::stream::Error::Broken(StreamBrokenError::Recv))?
                .map_err(|e| super::stream::Error::Alive(e.into()))?,
        )
        .map_err(|e| super::stream::Error::Alive(StreamAliveError::Parsing(e)))
    }

    async fn send(&self, msg: O) -> Result<(), Self::BrokenError> {
        self.sender
            .send(msg.into())
            .map_err(StreamBrokenError::Send)
    }

    type AliveError = StreamAliveError<AE>;
    type BrokenError = StreamBrokenError<SO>;
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

impl<I, O, SI, SO, S, AE> super::stream::Stream<I, O> for ServerStream<SI, SO, S>
where
    I: TryFrom<SI, Error = AE> + Send,
    O: Into<SO> + Send,
    SO: Send + Sync + 'static + std::fmt::Debug,
    S: futures::Stream<Item = Result<SI, Status>> + Unpin + Send,
    AE: std::error::Error,
{
    async fn recv(&self) -> Result<I, super::stream::Error<Self::AliveError, Self::BrokenError>> {
        I::try_from(
            self.receiver
                .lock()
                .await
                .next()
                .await
                .ok_or(super::stream::Error::Broken(StreamBrokenError::Recv))?
                .map_err(|e| super::stream::Error::Alive(e.into()))?,
        )
        .map_err(|e| super::stream::Error::Alive(StreamAliveError::Parsing(e)))
    }

    async fn send(&self, msg: O) -> Result<(), Self::BrokenError> {
        Ok(self.sender.send(Ok(msg.into()))?)
    }

    type AliveError = StreamAliveError<AE>;
    type BrokenError = StreamBrokenError<Result<SO, Status>>;
}
