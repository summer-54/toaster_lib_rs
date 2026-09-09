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
