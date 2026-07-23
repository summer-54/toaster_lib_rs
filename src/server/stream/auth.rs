use crate::auth::{Challenge, Solution};

use crate::logger::short_slice;

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
