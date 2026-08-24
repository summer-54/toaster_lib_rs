pub const NAME: &str = "master";

#[allow(dead_code)]
pub enum ManagerToInvoker {
    Close,
}

impl std::fmt::Debug for ManagerToInvoker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Close => write!(f, "Close"),
        }
    }
}

#[allow(dead_code)]
pub enum InvokerToManager {
    Exited { code: u8, data: Box<str> },
}

impl std::fmt::Debug for InvokerToManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exited { code, data } => f
                .debug_struct("Exited")
                .field("code", code)
                .field("data", data)
                .finish(),
        }
    }
}
