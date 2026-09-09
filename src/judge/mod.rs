use serde::{Deserialize, Serialize};
use std::ops::Deref;

pub mod lang;
pub mod submission;
pub mod test;

pub use lang::Lang;

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
