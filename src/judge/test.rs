use std::fmt::Debug;

pub mod verdict {
    use serde::{Deserialize, Serialize};
    use std::{fmt::Debug, str::FromStr};

    #[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
    pub enum Verdict {
        Ok, //ok
        Wa, //wrong answer
        Pe, //presentation error
        Ml, //memory limit
        Tl, //time limit
        Re, //runtime error
        Ce, //compile error
        Te, //testing system error
        Sl, //stack limit
    }

    impl Verdict {
        pub fn is_success(&self) -> bool {
            *self == Verdict::Ok
        }
    }

    impl std::fmt::Display for Verdict {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Verdict::Ok => "OK",
                    Verdict::Wa => "WA",
                    Verdict::Pe => "PE",
                    Verdict::Ml => "ML",
                    Verdict::Tl => "TL",
                    Verdict::Re => "RE",
                    Verdict::Ce => "CE",
                    Verdict::Te => "TE",
                    Verdict::Sl => "SL",
                }
            )
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    #[error("provided string was not valid verdict value")]
    pub struct ParseError;

    impl FromStr for Verdict {
        type Err = ParseError;
        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            Ok(match s {
                "OK" => Self::Ok,
                "WA" => Self::Wa,
                "PE" => Self::Pe,
                "ML" => Self::Ml,
                "TL" => Self::Tl,
                "RE" => Self::Re,
                "CE" => Self::Ce,
                "TE" => Self::Te,
                "SL" => Self::Sl,
                _ => return Err(ParseError),
            })
        }
    }
}

pub use verdict::Verdict;

#[derive(Clone)]
pub struct Result {
    pub verdict: Verdict,
    pub time: f64,
    pub memory: u64,
}

impl Debug for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Result")
            .field("verdict", &self.verdict)
            .field("time", &self.time)
            .field("memory", &self.memory)
            .finish()
    }
}

pub struct ResultPayload {
    pub result: Result,
    pub id: usize,
    pub data: Box<[u8]>,
}
