use std::{fmt::Display, ops::Deref};

use crate::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[serde(rename = "g++")]
    Gpp,
    #[serde(rename = "python3")]
    Python,
}

impl TryFrom<&str> for Lang {
    type Error = Error;
    fn try_from(s: &str) -> Result<Self> {
        match &*s.to_lowercase() {
            "g++" => Ok(Lang::Gpp),
            "python3" => Ok(Lang::Python),
            _ => bail!("unknown language: {}", s),
        }
    }
}

impl Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Gpp => "g++",
                Self::Python => "python3",
            }
        )
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ErrorMsg(Box<str>);

impl ErrorMsg {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> Box<str> {
        self.0
    }
}

impl Deref for ErrorMsg {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<T: Into<Box<str>>> From<T> for ErrorMsg {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

pub mod test {
    use crate::prelude::*;
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

    impl FromStr for Verdict {
        type Err = Error;

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
                verdict => {
                    bail!("incorrect verdict {}", verdict.bold())
                }
            })
        }
    }

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
}

pub mod submission {
    use std::ops::Deref;

    use uuid::Uuid;

    use crate::judge::test;

    #[repr(transparent)]
    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct SubmissionId(Uuid);

    impl SubmissionId {
        pub fn into_inner(self) -> Uuid {
            self.0
        }

        pub fn new(s: Uuid) -> Self {
            Self(s)
        }
    }

    impl Deref for SubmissionId {
        type Target = Uuid;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: Into<Uuid>> From<T> for SubmissionId {
        fn from(value: T) -> Self {
            Self(value.into())
        }
    }

    #[derive(Debug, Clone)]

    pub enum ResultWrapper<T> {
        Ok {
            score: usize,
            group_scores: Box<[usize]>,
            value: T,
        },
        Ce(Box<str>),
        Te(Box<str>),
    }

    pub type Result = ResultWrapper<()>;
    pub type FullResult = ResultWrapper<Box<[Option<test::Result>]>>;
}
