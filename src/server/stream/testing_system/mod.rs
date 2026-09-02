use crate::{
    judge::{Lang, submission},
    logger::short_slice,
};

pub enum SystemToManager {
    Judge {
        submission_id: submission::SubmissionId,
        test_count: usize,
        lang: Lang,
        data: Box<[u8]>,
    },
}

impl std::fmt::Debug for SystemToManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Judge {
                submission_id,
                test_count,
                lang,
                data,
            } => f
                .debug_struct("Judge")
                .field("submission_id", submission_id)
                .field("test_count", test_count)
                .field("lang", lang)
                .field("data", &Box::<[u8]>::from(short_slice(data)))
                .finish(),
        }
    }
}

pub enum ManagerToSystem {
    SubmissionResult {
        submission_id: submission::SubmissionId,
        result: submission::FullResult,
    },
    TestData {
        submission_id: submission::SubmissionId,
        test_id: usize,
        data: Box<[u8]>,
    },
}

impl std::fmt::Debug for ManagerToSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SubmissionResult {
                result,
                submission_id,
            } => f
                .debug_struct("SubmissionResult")
                .field("submission_id", submission_id)
                .field("result", result)
                .finish(),
            Self::TestData {
                test_id,
                data,
                submission_id,
            } => f
                .debug_struct("TestData")
                .field("submission_id", submission_id)
                .field("test_id", test_id)
                .field("data", &Box::<[u8]>::from(short_slice(data)))
                .finish(),
        }
    }
}
