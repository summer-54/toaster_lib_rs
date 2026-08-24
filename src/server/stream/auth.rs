use crate::auth::{Challenge, Solution};

use crate::logger::short_slice;

pub const NAME: &str = "auth";

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
    AuthProof(Solution),
    CertName(Box<str>),
}

impl std::fmt::Debug for InvokerToManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthProof(data) => f
                .debug_struct("AuthProof")
                .field("data", &Box::<[u8]>::from(short_slice(data)))
                .finish(),
            Self::CertName(name) => f.debug_struct("CertName").field("data", &name).finish(),
        }
    }
}
