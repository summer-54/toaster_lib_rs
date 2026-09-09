use uuid::Uuid;

use crate::judge::submission;

use super::super::pb::testing_system as ts_pb;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("cannot parse submission id")]
pub struct ParseError(#[from] uuid::Error);

impl TryFrom<ts_pb::SubmissionId> for submission::SubmissionId {
    type Error = ParseError;

    fn try_from(proto: ts_pb::SubmissionId) -> Result<Self, Self::Error> {
        Ok(Self::new(Uuid::from_slice(&proto.id).map_err(ParseError)?))
    }
}

impl From<submission::SubmissionId> for ts_pb::SubmissionId {
    fn from(id: submission::SubmissionId) -> Self {
        Self {
            id: Vec::from(id.as_bytes()),
        }
    }
}
