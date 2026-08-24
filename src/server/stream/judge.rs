use crate::{
    judge::{Lang, submission, test},
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
    FullResult(submission::Result),
    TestResult(test::ResultPayload),
    Error { msg: Box<str> },
    OpError { msg: Box<str> },
}

impl std::fmt::Debug for InvokerToManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FullResult(result) => f.debug_tuple("FullResult").field(result).finish(),
            Self::TestResult(test::ResultPayload { id, result, data }) => f
                .debug_struct("TestVerdict")
                .field("id", id)
                .field("result", result)
                .field("data", &Box::<[u8]>::from(short_slice(data)))
                .finish(),
            Self::Error { msg } => f.debug_struct("Error").field("msg", msg).finish(),
            Self::OpError { msg } => f.debug_struct("OpError").field("msg", msg).finish(),
        }
    }
}
