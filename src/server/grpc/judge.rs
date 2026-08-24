use super::{
    super::stream::judge::{InvokerToManager, ManagerToInvoker},
    pb,
};
use crate::{
    judge::{Lang, submission, test},
    prelude::*,
};
impl TryInto<InvokerToManager> for pb::JudgeIncome {
    type Error = Error;
    fn try_into(self) -> Result<InvokerToManager> {
        let Some(payload) = self.payload else {
            bail!("payload not found");
        };
        Ok(match payload {
            pb::judge_income::Payload::TestResult(result) => {
                InvokerToManager::TestResult(test::ResultPayload {
                    result: test::Result {
                        verdict: match result.verdict() {
                            pb::TestVerdict::VerdictUnspecified => bail!("verdict unspecified"),
                            pb::TestVerdict::Ok => test::Verdict::Ok,
                            pb::TestVerdict::Wa => test::Verdict::Wa,
                            pb::TestVerdict::Tl => test::Verdict::Tl,
                            pb::TestVerdict::Ml => test::Verdict::Ml,
                            pb::TestVerdict::Sl => test::Verdict::Sl,
                            pb::TestVerdict::Re => test::Verdict::Re,
                            pb::TestVerdict::Ce => test::Verdict::Ce,
                            pb::TestVerdict::Te => test::Verdict::Te,
                            pb::TestVerdict::Pe => test::Verdict::Pe,
                        },
                        time: result.time_sec,
                        memory: result.memory_bytes,
                    },

                    id: result.id as usize,
                    data: result.data.into(),
                })
            }
            pb::judge_income::Payload::FullResult(result) => {
                let verdict = result.verdict();
                let details = result.details.context("reading detailes")?;
                InvokerToManager::FullResult(match verdict {
                    pb::FullVerdict::Unspecified => bail!("verdict uncpecified"),
                    pb::FullVerdict::FullOk => {
                        let pb::full_result::Details::Success(details) = details else {
                            bail!("details not success while verdct is success")
                        };
                        submission::Result::Ok {
                            score: details.sum as usize,
                            groups_score: details
                                .groups
                                .into_iter()
                                .map(|score| score as usize)
                                .collect(),
                        }
                    }
                    pb::FullVerdict::FullCe => {
                        let pb::full_result::Details::Error(details) = details else {
                            bail!("details is success while verdct is not success")
                        };

                        submission::Result::Ce(details.message.into())
                    }
                    pb::FullVerdict::FullTe => {
                        let pb::full_result::Details::Error(details) = details else {
                            bail!("details is success while verdct is not success")
                        };

                        submission::Result::Te(details.message.into())
                    }
                })
            }
            pb::judge_income::Payload::Error(invoker_error) => InvokerToManager::Error {
                msg: invoker_error.message.into(),
            },
            pb::judge_income::Payload::Operror(operator_error) => InvokerToManager::OpError {
                msg: operator_error.message.into(),
            },
        })
    }
}

impl From<InvokerToManager> for pb::JudgeIncome {
    fn from(value: InvokerToManager) -> Self {
        Self {
            payload: Some(match value {
                InvokerToManager::FullResult(result) => {
                    pb::judge_income::Payload::FullResult(match result {
                        submission::Result::Ok {
                            score,
                            groups_score,
                        } => pb::FullResult {
                            verdict: pb::FullVerdict::FullOk.into(),
                            details: Some(pb::full_result::Details::Success(
                                pb::full_result::SuccessDetails {
                                    sum: score as u32,
                                    groups: groups_score
                                        .into_iter()
                                        .map(|score| score as u32)
                                        .collect(),
                                },
                            )),
                        },
                        submission::Result::Ce(msg) => pb::FullResult {
                            verdict: pb::FullVerdict::FullCe.into(),
                            details: Some(pb::full_result::Details::Error(
                                pb::full_result::ErrorDetails {
                                    message: msg.into(),
                                },
                            )),
                        },
                        submission::Result::Te(msg) => pb::FullResult {
                            verdict: pb::FullVerdict::FullTe.into(),
                            details: Some(pb::full_result::Details::Error(
                                pb::full_result::ErrorDetails {
                                    message: msg.into(),
                                },
                            )),
                        },
                    })
                }
                InvokerToManager::TestResult(result_payload) => {
                    pb::judge_income::Payload::TestResult(pb::TestResult {
                        id: result_payload.id as u32,
                        verdict: match result_payload.result.verdict {
                            test::Verdict::Ok => pb::TestVerdict::Ok,
                            test::Verdict::Wa => pb::TestVerdict::Wa,
                            test::Verdict::Pe => pb::TestVerdict::Pe,
                            test::Verdict::Ml => pb::TestVerdict::Ml,
                            test::Verdict::Tl => pb::TestVerdict::Tl,
                            test::Verdict::Re => pb::TestVerdict::Re,
                            test::Verdict::Ce => pb::TestVerdict::Ce,
                            test::Verdict::Te => pb::TestVerdict::Te,
                            test::Verdict::Sl => pb::TestVerdict::Sl,
                        }
                        .into(),
                        time_sec: result_payload.result.time,
                        memory_bytes: result_payload.result.memory,
                        data: result_payload.data.into(),
                    })
                }

                InvokerToManager::Error { msg } => {
                    pb::judge_income::Payload::Error(pb::InvokerError {
                        message: msg.into(),
                    })
                }
                InvokerToManager::OpError { msg } => {
                    pb::judge_income::Payload::Operror(pb::OperatorError {
                        message: msg.into(),
                    })
                }
            }),
        }
    }
}

impl TryInto<ManagerToInvoker> for pb::JudgeOutgo {
    type Error = Error;
    fn try_into(self) -> Result<ManagerToInvoker> {
        let Some(payload) = self.payload else {
            bail!("payload not found");
        };
        Ok(match payload {
            pb::judge_outgo::Payload::Run(run_task) => ManagerToInvoker::Run {
                lang: match run_task.lang() {
                    pb::Language::LangUnspecified => bail!("lang unspecified"),
                    pb::Language::LangGpp => Lang::Gpp,
                    pb::Language::LangPython3 => Lang::Python,
                },
                data: run_task.data.into(),
            },
            pb::judge_outgo::Payload::Stop(_) => ManagerToInvoker::Stop,
        })
    }
}

impl From<ManagerToInvoker> for pb::JudgeOutgo {
    fn from(value: ManagerToInvoker) -> Self {
        Self {
            payload: Some(match value {
                ManagerToInvoker::Run { lang, data } => {
                    pb::judge_outgo::Payload::Run(pb::RunTask {
                        lang: match lang {
                            Lang::Gpp => pb::Language::LangGpp,
                            Lang::Python => pb::Language::LangPython3,
                        }
                        .into(),
                        data: data.into(),
                    })
                }
                ManagerToInvoker::Stop => pb::judge_outgo::Payload::Stop(pb::StopTask {}),
            }),
        }
    }
}
