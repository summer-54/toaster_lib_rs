use crate::{
    judge::{ErrorMsg, Lang, submission, test},
    logger::short_slice,
};

pub const NAME: &str = "judge";

#[allow(dead_code)]
pub enum ManagerToInvoker {
    Run { lang: Lang, data: Box<[u8]> },
    Stop,
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
        }
    }
}

#[allow(dead_code)]
pub enum InvokerToManager {
    SubmissionResult(submission::Result),
    TestResultPayload(test::ResultPayload),
    Error(ErrorMsg),
}

impl std::fmt::Debug for InvokerToManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SubmissionResult(result) => {
                f.debug_tuple("SubmissionResult").field(result).finish()
            }
            Self::TestResultPayload(test::ResultPayload { id, result, data }) => f
                .debug_struct("TestVerdict")
                .field("id", id)
                .field("result", result)
                .field("data", &Box::<[u8]>::from(short_slice(data)))
                .finish(),
            Self::Error(msg) => f.debug_struct("Error").field("msg", msg).finish(),
        }
    }
}
