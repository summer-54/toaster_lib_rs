use crate::prelude::*;

use crate::{
    judge::{Lang, submission, test},
    logger::short_slice,
};

pub const NAME: &str = "MASTER";

use super::{MappedRawMessage, RawMessage};

#[allow(dead_code)]
pub enum ManagerToInvoker {
    Run { lang: Lang, data: Box<[u8]> },
    Stop,
    Close,
}

impl std::fmt::Debug for ManagerToInvoker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Run { lang, data } => f
                .debug_struct("Start")
                .field("lang", lang)
                .field("data", &Box::<[u8]>::from(short_slice(data)))
                .finish(),
            Self::Stop => write!(f, "Stop"),
            Self::Close => write!(f, "Close"),
        }
    }
}

impl super::Income for ManagerToInvoker {
    fn from_raw(msg: MappedRawMessage) -> Result<Self> {
        Ok(match msg.ty() {
            "START" => {
                let Some(data) = msg.data() else {
                    bail!("{} not found", "data".bold());
                };
                let Some(lang) = msg.field("LANG") else {
                    bail!("{} field not found", "LANG".bold());
                };

                Self::Run {
                    lang: Lang::try_from(lang).context("parsing lang")?,
                    data: Box::from(data),
                }
            }
            "STOP" => Self::Stop,
            "CLOSE" => Self::Close,
            command => {
                bail!("incorrect command '{}'", command.bold());
            }
        })
    }
}

impl super::Outgo for ManagerToInvoker {
    fn into_raw(self) -> RawMessage {
        match self {
            Self::Run { lang, data } => {
                let mut body = RawMessage::new("START");
                body.add_field(&"LANG", &lang);
                body.set_data(data);
                body
            }
            Self::Stop => RawMessage::new("STOP"),
            Self::Close => RawMessage::new("CLOSE"),
        }
    }
}

#[allow(dead_code)]
pub enum InvokerToManager {
    FullResult(submission::Result),
    TestResult {
        test_id: usize,
        result: test::Result,
        data: Box<[u8]>,
    },
    Error {
        msg: Box<str>,
    },
    OpError {
        msg: Box<str>,
    },
}

impl std::fmt::Debug for InvokerToManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FullResult(result) => f.debug_tuple("FullResult").field(result).finish(),
            Self::TestResult {
                test_id,
                result,
                data,
            } => f
                .debug_struct("TestVerdict")
                .field("test_id", test_id)
                .field("result", result)
                .field("data", &Box::<[u8]>::from(short_slice(data)))
                .finish(),
            Self::Error { msg } => f.debug_struct("Error").field("msg", msg).finish(),
            Self::OpError { msg } => f.debug_struct("OpError").field("msg", msg).finish(),
        }
    }
}
impl super::Income for InvokerToManager {
    fn from_raw(msg: MappedRawMessage) -> Result<Self> {
        Ok(match msg.ty() {
            "VERDICT" => InvokerToManager::FullResult(
                match msg
                    .field("NAME")
                    .ok_or(anyhow!("{} field not found", "NAME".bold()))?
                {
                    "OK" => {
                        let groups = msg
                            .field("GROUPS")
                            .ok_or(anyhow!("{} field not found", "GROUPS".bold()))?;
                        submission::Result::Ok {
                            score: msg
                                .field("SUM")
                                .ok_or(anyhow!("{} field not found", "SUM".bold()))?
                                .parse()
                                .context("parsing 'SUM' value")?,
                            groups_score: groups
                                .split_ascii_whitespace()
                                .map(|score| -> Result<usize> { Ok(score.parse()?) })
                                .collect::<Result<Box<[usize]>>>()
                                .context("parsing groups score")?,
                        }
                    }
                    "CE" => submission::Result::Ce(
                        msg.field("MESSAGE")
                            .ok_or(anyhow!("{} field not found", "MESSAGE".bold()))?
                            .into(),
                    ),
                    "TE" => submission::Result::Te(
                        msg.field("MESSAGE")
                            .ok_or(anyhow!("{} field not found", "MESSAGE".bold()))?
                            .into(),
                    ),
                    verdict => {
                        bail!("incorrect verdict '{}'", verdict.bold())
                    }
                },
            ),
            "TEST" => InvokerToManager::TestResult {
                test_id: msg
                    .field("ID")
                    .ok_or(anyhow!("{} field not found", "ID".bold()))?
                    .parse()
                    .context("parsing 'ID' field")?,
                result: test::Result {
                    verdict: msg
                        .field("NAME")
                        .ok_or(anyhow!("{} field not found", "NAME".bold()))?
                        .parse()
                        .context("parsing 'NAME' field")?,
                    time: msg
                        .field("TIME")
                        .ok_or(anyhow!("{} field not found", "TIME".bold()))?
                        .parse()
                        .context("parsing 'TIME' field")?,
                    memory: msg
                        .field("MEMORY")
                        .ok_or(anyhow!("{} field not found", "MEMORY".bold()))?
                        .parse()
                        .context("parsing 'MEMORY' field")?,
                },
                data: msg
                    .data()
                    .ok_or(anyhow!("{} not found", "data".bold()))?
                    .into(),
            },
            "ERROR" => InvokerToManager::Error {
                msg: msg
                    .field("MESSAGE")
                    .ok_or(anyhow!("{} not found", "MESSAGE".bold()))?
                    .into(),
            },
            "OPERROR" => InvokerToManager::OpError {
                msg: msg
                    .field("MESSAGE")
                    .ok_or(anyhow!("{} not found", "MESSAGE".bold()))?
                    .into(),
            },
            command => {
                bail!("incorrect command '{}'", command.bold());
            }
        })
    }
}
impl super::Outgo for InvokerToManager {
    fn into_raw(self) -> RawMessage {
        match self {
            Self::FullResult(verdict) => {
                let mut body = RawMessage::new("VERDICT");
                match verdict {
                    submission::Result::Ok {
                        score,
                        groups_score,
                    } => {
                        body.add_fields(vec![
                            (&"NAME", &"OK"),
                            (&"SUM", &score),
                            (
                                &"GROUPS",
                                &groups_score
                                    .into_iter()
                                    .map(|score| format!("{score}"))
                                    .collect::<Vec<_>>()
                                    .join(" "),
                            ),
                        ]);
                    }
                    submission::Result::Ce(msg) => {
                        body.add_fields(vec![(&"NAME", &"CE"), (&"MESSAGE", &msg)]);
                    }
                    submission::Result::Te(msg) => {
                        body.add_fields(vec![(&"NAME", &"TE"), (&"MESSAGE", &msg)]);
                    }
                }
                body
            }
            Self::TestResult {
                test_id,
                result,
                data,
            } => {
                let mut body = RawMessage::new("TEST");
                body.add_fields(vec![
                    (&"ID", &test_id),
                    (&"VERDICT", &result.verdict),
                    (&"TIME", &result.time),
                    (&"MEMORY", &result.memory),
                ])
                .set_data(data);
                body
            }
            Self::Error { msg } => {
                let mut body = RawMessage::new("ERROR");
                body.add_field(&"MESSAGE", &msg);
                body
            }
            Self::OpError { msg } => {
                let mut body = RawMessage::new("OPERROR");
                body.add_field(&"MESSAGE", &msg);
                body
            }
        }
    }
}
