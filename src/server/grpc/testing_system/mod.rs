use uuid::Uuid;

use super::{
    super::stream::testing_system::{ManagerToSystem, SystemToManager},
    pb::{testing_system as ts_pb, toaster as pb},
};
use crate::{
    judge::{
        self,
        submission::{self, FullResult},
        test,
    },
    prelude::*,
};

impl TryFrom<ts_pb::SubmissionId> for submission::SubmissionId {
    type Error = Error;

    fn try_from(proto: ts_pb::SubmissionId) -> Result<Self, Self::Error> {
        Ok(Self::new(
            Uuid::from_slice(&proto.id).context("parsing submission id")?,
        ))
    }
}

impl From<submission::SubmissionId> for ts_pb::SubmissionId {
    fn from(id: submission::SubmissionId) -> Self {
        Self {
            id: Vec::from(id.as_bytes()),
        }
    }
}

impl From<ManagerToSystem> for ts_pb::Income {
    fn from(domain: ManagerToSystem) -> Self {
        Self {
            payload: Some(match domain {
                ManagerToSystem::SubmissionResult {
                    submission_id,
                    result,
                } => ts_pb::income::Payload::SubmissionResult(match result {
                    crate::judge::submission::ResultWrapper::Ok {
                        score,
                        group_scores,
                        value,
                    } => ts_pb::SubmissionResult {
                        submission_id: Some(submission_id.into()),
                        verdict: pb::SubmissionVerdict::Ok.into(),
                        details: ts_pb::submission_result::Details::Success(
                            ts_pb::submission_result::SuccessDetails {
                                score: score as u32,
                                group_scores: group_scores.into_iter().map(|v| v as u32).collect(),
                                test_results: value
                                    .into_iter()
                                    .map(|result| {
                                        ts_pb::submission_result::success_details::TestResult {
                                            result: result.map(Into::into),
                                        }
                                    })
                                    .collect(),
                            },
                        )
                        .into(),
                    },
                    crate::judge::submission::ResultWrapper::Ce(message) => {
                        ts_pb::SubmissionResult {
                            submission_id: Some(submission_id.into()),
                            verdict: pb::SubmissionVerdict::Ce.into(),
                            details: ts_pb::submission_result::Details::Error(
                                ts_pb::submission_result::ErrorDetails {
                                    message: message.into(),
                                },
                            )
                            .into(),
                        }
                    }
                    crate::judge::submission::ResultWrapper::Te(message) => {
                        ts_pb::SubmissionResult {
                            submission_id: Some(submission_id.into()),
                            verdict: pb::SubmissionVerdict::Te.into(),
                            details: ts_pb::submission_result::Details::Error(
                                ts_pb::submission_result::ErrorDetails {
                                    message: message.into(),
                                },
                            )
                            .into(),
                        }
                    }
                }),
                ManagerToSystem::TestData {
                    submission_id,
                    test_id,
                    data,
                } => ts_pb::income::Payload::TestData(ts_pb::TestData {
                    submission_id: Some(submission_id.into()),
                    test_id: test_id as u32,
                    data: data.into(),
                }),
            }),
        }
    }
}

impl TryFrom<ts_pb::Income> for ManagerToSystem {
    type Error = Error;

    fn try_from(proto: ts_pb::Income) -> Result<Self, Self::Error> {
        let Some(payload) = proto.payload else {
            bail!("payload not found");
        };

        Ok(match payload {
            ts_pb::income::Payload::TestData(test_data) => Self::TestData {
                submission_id: test_data
                    .submission_id
                    .context("reading submission id")?
                    .try_into()
                    .context("parsing submission id")?,
                test_id: test_data.test_id as usize,
                data: test_data.data.into(),
            },
            ts_pb::income::Payload::SubmissionResult(submission_result) => {
                match submission_result.verdict() {
                    pb::SubmissionVerdict::Unspecified => bail!("submission verdict unspecified"),
                    pb::SubmissionVerdict::Ok => {
                        let ts_pb::submission_result::Details::Success(detailes) =
                            submission_result.details.context("parsing details")?
                        else {
                            bail!("details incorrect")
                        };
                        Self::SubmissionResult {
                            submission_id: submission_result
                                .submission_id
                                .context("reading submission id")?
                                .try_into()
                                .context("parsing submission id")?,
                            result: FullResult::Ok {
                                score: detailes.score as usize,
                                group_scores: detailes
                                    .group_scores
                                    .into_iter()
                                    .map(|score| score as usize)
                                    .collect(),
                                value: detailes
                                    .test_results
                                    .into_iter()
                                    .map(|result| -> Result<Option<test::Result>> {
                                        let Some(result) = result.result else {
                                            return Ok(None);
                                        };
                                        Ok(Some(result.try_into()?))
                                    })
                                    .collect::<Result<Box<[Option<test::Result>]>>>()?,
                            },
                        }
                    }
                    pb::SubmissionVerdict::Ce => {
                        let ts_pb::submission_result::Details::Error(details) =
                            submission_result.details.context("parsing details")?
                        else {
                            bail!("expected error details for CE verdict")
                        };
                        Self::SubmissionResult {
                            submission_id: submission_result
                                .submission_id
                                .context("reading submission id")?
                                .try_into()
                                .context("parsing submission id")?,
                            result: FullResult::Ce(details.message.into()),
                        }
                    }
                    pb::SubmissionVerdict::Te => {
                        let ts_pb::submission_result::Details::Error(details) =
                            submission_result.details.context("parsing details")?
                        else {
                            bail!("expected error details for TE verdict")
                        };
                        Self::SubmissionResult {
                            submission_id: submission_result
                                .submission_id
                                .context("reading submission id")?
                                .try_into()
                                .context("parsing submission id")?,
                            result: FullResult::Te(details.message.into()),
                        }
                    }
                }
            }
        })
    }
}

impl From<SystemToManager> for ts_pb::Outgo {
    fn from(domain: SystemToManager) -> Self {
        Self {
            payload: Some(match domain {
                SystemToManager::Judge {
                    submission_id,
                    test_count,
                    lang,
                    data,
                } => ts_pb::outgo::Payload::JudgeSubmission(ts_pb::JudgeSubmission {
                    submission_id: Some(submission_id.into()),
                    tests_count: test_count as u32,
                    lang: <judge::Lang as Into<pb::Lang>>::into(lang).into(),
                    data: data.into_vec(),
                }),
            }),
        }
    }
}

impl TryFrom<ts_pb::Outgo> for SystemToManager {
    type Error = Error;

    fn try_from(proto: ts_pb::Outgo) -> Result<Self, Self::Error> {
        let Some(payload) = proto.payload else {
            bail!("payload not found");
        };

        Ok(match payload {
            ts_pb::outgo::Payload::JudgeSubmission(judge) => Self::Judge {
                submission_id: judge
                    .submission_id
                    .context("reading submission id")?
                    .try_into()
                    .context("parsing submission id")?,
                test_count: judge.tests_count as usize,
                lang: pb::Lang::try_from(judge.lang)
                    .context("parsing lang")?
                    .try_into()
                    .context("converting lang")?,
                data: judge.data.into(),
            },
        })
    }
}
