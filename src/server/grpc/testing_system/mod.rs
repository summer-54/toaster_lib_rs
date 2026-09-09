pub mod submission_id;

pub use ts_pb::{
    testing_system_service_client::TestingSystemServiceClient as Client,
    testing_system_service_server::{
        TestingSystemService as Service, TestingSystemServiceServer as Server,
    },
};

use super::{
    super::stream::testing_system::{ManagerToSystem, SystemToManager},
    ConvertError, FieldContext, convert_field, convert_unwrap_field,
    pb::{testing_system as ts_pb, toaster as pb},
    unwrap_field,
};
use crate::judge::{self, submission::FullResult, test};

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
    type Error = ConvertError;

    fn try_from(proto: ts_pb::Income) -> Result<Self, Self::Error> {
        (|| {
            let Some(payload) = proto.payload else {
                return Err(ConvertError::FieldUndefined);
            };
            Ok(match payload {
                ts_pb::income::Payload::TestData(test_data) => {
                    let submission_id = convert_unwrap_field("submission_id", test_data.submission_id)?;
                    Self::TestData {
                        submission_id,
                        test_id: test_data.test_id as usize,
                        data: test_data.data.into(),
                    }
                }
                ts_pb::income::Payload::SubmissionResult(submission_result) => {
                    match submission_result.verdict() {
                        pb::SubmissionVerdict::Unspecified => {
                            return Err(ConvertError::EnumUnspecified.at_field("verdict"));
                        }
                        pb::SubmissionVerdict::Ok => {
                            let ts_pb::submission_result::Details::Success(detailes) =
                                unwrap_field("detailes", submission_result.details)?
                                else {
                                    return Err(ConvertError::Other("details incorrect".into()));
                                };
                            let submission_id =
                                convert_unwrap_field("submission_id", submission_result.submission_id)?;
                            Self::SubmissionResult {
                                submission_id,
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
                                        .map(
                                            |result| -> Result<Option<test::Result>, ConvertError> {
                                                let Some(result) = result.result else {
                                                    return Ok(None);
                                                };
                                                Ok(Some(result.try_into()?))
                                            },
                                        )
                                        .collect::<Result<Box<[Option<test::Result>]>, ConvertError>>().map_err(|e| e.at_field("test_results"))?,
                                },
                            }
                        }
                        pb::SubmissionVerdict::Ce => {
                            let ts_pb::submission_result::Details::Error(details) =
                                unwrap_field("detailes",submission_result.details)?
                                else {
                                    return Err(ConvertError::Other("expected error details for CE verdict".into()))
                                };
                            let submission_id =
                                convert_unwrap_field("submission_id", submission_result.submission_id)?;
                            Self::SubmissionResult {
                                submission_id,
                                result: FullResult::Ce(details.message.into()),
                            }
                        }
                        pb::SubmissionVerdict::Te => {
                            let ts_pb::submission_result::Details::Error(details) = unwrap_field("detailes",submission_result.details)?
                                else {
                                    return Err(ConvertError::Other("expected error details for TE verdict".into()))
                                };
                            let submission_id =
                                convert_unwrap_field("submission_id", submission_result.submission_id)?;
                            Self::SubmissionResult {
                                submission_id,
                                result: FullResult::Te(details.message.into()),
                            }
                        }
                    }
                }
            })
        })()
        .at_field("payload")
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
    type Error = ConvertError;

    fn try_from(proto: ts_pb::Outgo) -> Result<Self, Self::Error> {
        (|| {
            let Some(payload) = proto.payload else {
                return Err(ConvertError::FieldUndefined);
            };

            Ok(match payload {
                ts_pb::outgo::Payload::JudgeSubmission(judge) => {
                    let lang = convert_field("lang", judge.lang())?;
                    Self::Judge {
                        submission_id: convert_unwrap_field("submission_id", judge.submission_id)?,
                        test_count: judge.tests_count as usize,
                        lang,
                        data: judge.data.into(),
                    }
                }
            })
        })()
        .at_field("payload")
    }
}
