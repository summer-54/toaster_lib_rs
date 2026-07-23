use crate::prelude::*;

use crate::auth::{Challenge, Solution};

use crate::logger::short_slice;

use super::{MappedRawMessage, RawMessage};
pub const NAME: &str = "AUTH";

#[allow(dead_code)]
pub enum ManagerToInvoker {
    Challenge(Challenge),
    Verdict(bool),
}
impl std::fmt::Debug for ManagerToInvoker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verdict(verdict) => {
                write!(f, "{}", if *verdict { "Approved" } else { "Denied" })
            }
            Self::Challenge(challenge) => f
                .debug_struct("Challenge")
                .field("data", &Box::<[u8]>::from(short_slice(challenge)))
                .finish(),
        }
    }
}
impl super::Income for ManagerToInvoker {
    fn from_raw(msg: MappedRawMessage) -> Result<Self> {
        Ok(match msg.ty() {
            "VERDICT" => Self::Verdict(msg.field_eq("VERDICT", "APPROVED")),
            "CHALLENGE" => {
                let Some(data) = msg.data() else {
                    bail!("data not found");
                };
                Self::Challenge(Challenge::from(data))
            }
            command => {
                bail!("incorrect command '{}'", command.bold());
            }
        })
    }
}

impl super::Outgo for ManagerToInvoker {
    fn into_raw(self) -> RawMessage {
        match self {
            Self::Challenge(data) => {
                let mut body = RawMessage::new("CHALLENGE");
                body.set_data(Box::from(&*data));
                body
            }
            ManagerToInvoker::Verdict(is_approved) => {
                let mut body = RawMessage::new("VERDICT");
                body.add_field(&"VERDICT", &if is_approved { "APPROVED" } else { "DENIED" });
                body
            }
        }
    }
}

pub enum InvokerToManager {
    ChallengeSolution(Solution),
}

impl std::fmt::Debug for InvokerToManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChallengeSolution(data) => f
                .debug_struct("ChallengeSolution")
                .field("data", &Box::<[u8]>::from(short_slice(data)))
                .finish(),
        }
    }
}
impl super::Income for InvokerToManager {
    fn from_raw(msg: MappedRawMessage) -> Result<Self> {
        Ok(match msg.ty() {
            "PROOF" => Self::ChallengeSolution(
                msg.data()
                    .ok_or(anyhow!("{} not found", "data".bold()))?
                    .into(),
            ),
            command => {
                bail!("incorrect command '{}'", command.bold());
            }
        })
    }
}
impl super::Outgo for InvokerToManager {
    fn into_raw(self) -> RawMessage {
        match self {
            Self::ChallengeSolution(data) => {
                let mut body = RawMessage::new("PROOF");
                body.set_data(Box::from(&*data));
                body
            }
        }
    }
}
