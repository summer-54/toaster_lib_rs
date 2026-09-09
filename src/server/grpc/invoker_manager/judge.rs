use super::super::{
    super::stream::invoker_manager::judge::{InvokerToManager, ManagerToInvoker},
    ConvertError, FieldContext, convert_unwrap_field,
    pb::invoker_manager as im_pb,
    pb::toaster as pb,
    unwrap_field,
};
use crate::judge::{ErrorMsg, Lang, submission, test};

impl TryFrom<im_pb::SubmissionResult> for submission::Result {
    type Error = ConvertError;

    fn try_from(proto: im_pb::SubmissionResult) -> Result<Self, Self::Error> {
        let verdict = proto.verdict();
        let details = unwrap_field("details", proto.details)?;
        (|| {
            Ok(match verdict {
                pb::SubmissionVerdict::Unspecified => {
                    return Err(ConvertError::EnumUnspecified.at_field("verdict"));
                }
                pb::SubmissionVerdict::Ok => {
                    let im_pb::submission_result::Details::Success(details) = details else {
                        return Err(ConvertError::Other(
                            "details not success while verdct is success".into(),
                        ));
                    };
                    Self::Ok {
                        score: details.score as usize,
                        group_scores: details
                            .group_scores
                            .into_iter()
                            .map(|score| score as usize)
                            .collect(),
                        value: (),
                    }
                }
                pb::SubmissionVerdict::Ce => {
                    let im_pb::submission_result::Details::Error(details) = details else {
                        return Err(ConvertError::Other(
                            "details is success while verdct is not success".into(),
                        ));
                    };
                    Self::Ce(details.message.into())
                }
                pb::SubmissionVerdict::Te => {
                    let im_pb::submission_result::Details::Error(details) = details else {
                        return Err(ConvertError::Other(
                            "details is success while verdct is not success".into(),
                        ));
                    };
                    Self::Te(details.message.into())
                }
            })
        })()
        .at_field("verdict")
    }
}

impl From<submission::Result> for im_pb::SubmissionResult {
    fn from(result: submission::Result) -> Self {
        match result {
            submission::Result::Ok {
                score,
                group_scores,
                ..
            } => im_pb::SubmissionResult {
                verdict: pb::SubmissionVerdict::Ok.into(),
                details: Some(im_pb::submission_result::Details::Success(
                    im_pb::submission_result::SuccessDetails {
                        score: score as u32,
                        group_scores: group_scores.into_iter().map(|score| score as u32).collect(),
                    },
                )),
            },
            submission::Result::Ce(msg) => im_pb::SubmissionResult {
                verdict: pb::SubmissionVerdict::Ce.into(),
                details: Some(im_pb::submission_result::Details::Error(
                    im_pb::submission_result::ErrorDetails {
                        message: msg.into(),
                    },
                )),
            },
            submission::Result::Te(msg) => im_pb::SubmissionResult {
                verdict: pb::SubmissionVerdict::Te.into(),
                details: Some(im_pb::submission_result::Details::Error(
                    im_pb::submission_result::ErrorDetails {
                        message: msg.into(),
                    },
                )),
            },
        }
    }
}

impl TryFrom<im_pb::TestResultPayload> for test::ResultPayload {
    type Error = ConvertError;

    fn try_from(proto: im_pb::TestResultPayload) -> Result<Self, Self::Error> {
        Ok(test::ResultPayload {
            result: convert_unwrap_field("result", proto.result)?,
            id: proto.id as usize,
            data: proto.data.into(),
        })
    }
}

impl From<test::ResultPayload> for im_pb::TestResultPayload {
    fn from(payload: test::ResultPayload) -> Self {
        Self {
            id: payload.id as u32,
            result: Some(payload.result.into()),
            data: payload.data.into(),
        }
    }
}
impl TryFrom<im_pb::Error> for ErrorMsg {
    type Error = ConvertError;

    fn try_from(proto: im_pb::Error) -> Result<Self, Self::Error> {
        Ok(proto.message.into())
    }
}

impl From<ErrorMsg> for im_pb::Error {
    fn from(msg: ErrorMsg) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}

impl TryFrom<im_pb::JudgeIncome> for InvokerToManager {
    type Error = ConvertError;

    fn try_from(
        proto: im_pb::JudgeIncome,
    ) -> Result<Self, <InvokerToManager as TryFrom<im_pb::JudgeIncome>>::Error> {
        let payload = unwrap_field("payload", proto.payload)?;
        Ok(match payload {
            im_pb::judge_income::Payload::TestResultPayload(result) => {
                Self::TestResultPayload(test::ResultPayload::try_from(result)?)
            }
            im_pb::judge_income::Payload::SubmissionResult(result) => {
                Self::SubmissionResult(submission::Result::try_from(result)?)
            }
            im_pb::judge_income::Payload::Error(invoker_error) => {
                Self::Error(ErrorMsg::try_from(invoker_error)?)
            }
        })
    }
}

impl From<InvokerToManager> for im_pb::JudgeIncome {
    fn from(msg: InvokerToManager) -> Self {
        Self {
            payload: Some(match msg {
                InvokerToManager::SubmissionResult(result) => {
                    im_pb::judge_income::Payload::SubmissionResult(result.into())
                }
                InvokerToManager::TestResultPayload(result_payload) => {
                    im_pb::judge_income::Payload::TestResultPayload(result_payload.into())
                }
                InvokerToManager::Error(msg) => im_pb::judge_income::Payload::Error(msg.into()),
            }),
        }
    }
}

impl TryFrom<im_pb::JudgeOutgo> for ManagerToInvoker {
    type Error = ConvertError;

    fn try_from(proto: im_pb::JudgeOutgo) -> Result<Self, Self::Error> {
        let payload = unwrap_field("payload", proto.payload)?;
        Ok(match payload {
            im_pb::judge_outgo::Payload::Run(run_task) => ManagerToInvoker::Run {
                lang: Lang::try_from(run_task.lang())?,
                data: run_task.data.into(),
            },
            im_pb::judge_outgo::Payload::Stop(_) => ManagerToInvoker::Stop,
        })
    }
}

impl From<ManagerToInvoker> for im_pb::JudgeOutgo {
    fn from(proto: ManagerToInvoker) -> Self {
        Self {
            payload: Some(match proto {
                ManagerToInvoker::Run { lang, data } => {
                    im_pb::judge_outgo::Payload::Run(im_pb::RunTask {
                        lang: pb::Lang::from(lang).into(),
                        data: data.into(),
                    })
                }
                ManagerToInvoker::Stop => im_pb::judge_outgo::Payload::Stop(im_pb::StopTask {}),
            }),
        }
    }
}
