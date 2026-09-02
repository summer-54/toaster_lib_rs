pub const NAME: &str = "master";

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
