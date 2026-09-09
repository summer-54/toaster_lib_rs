use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("provided string was not valid lang value")]
pub struct ParseError;
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[serde(rename = "g++")]
    Gpp,
    #[serde(rename = "python3")]
    Python,
}

impl FromStr for Lang {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_lowercase() {
            "g++" => Ok(Lang::Gpp),
            "python3" => Ok(Lang::Python),
            _ => Err(ParseError),
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
