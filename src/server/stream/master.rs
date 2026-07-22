use uuid::Uuid;

use crate::prelude::*;

pub const NAME: &str = "MASTER";

use super::{MappedRawMessage, RawMessage};

#[allow(dead_code)]
pub enum ManagerToInvoker {
    Stop,
    Close,
}

impl std::fmt::Debug for ManagerToInvoker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stop => write!(f, "Stop"),
            Self::Close => write!(f, "Close"),
        }
    }
}

impl super::Income for ManagerToInvoker {
    fn from_raw(msg: MappedRawMessage) -> Result<Self> {
        Ok(match msg.ty() {
            "STOP" => Self::Stop,
            "CLOSE" => Self::Close,
            command => {
                bail!("incorrect command '{}'", command.bold());
            }
        })
    }
}

impl super::Outgo for ManagerToInvoker {
    fn into_raw(self) -> RawMessage {
        match self {
            Self::Stop => RawMessage::new("STOP"),
            Self::Close => RawMessage::new("CLOSE"),
        }
    }
}

#[allow(dead_code)]
pub enum InvokerToManager {
    Token { token: uuid::Uuid, name: Box<str> },
    Exited { code: u8, data: Box<str> },
}

impl std::fmt::Debug for InvokerToManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Token { token, name } => f
                .debug_struct("Token")
                .field("token", token)
                .field("name", name)
                .finish(),

            Self::Exited { code, data } => f
                .debug_struct("Exited")
                .field("code", code)
                .field("data", data)
                .finish(),
        }
    }
}
impl super::Income for InvokerToManager {
    fn from_raw(msg: MappedRawMessage) -> Result<Self> {
        Ok(match msg.ty() {
            "TOKEN" => InvokerToManager::Token {
                token: msg
                    .field("ID")
                    .ok_or(anyhow!("{} not found", "ID".bold()))?
                    .parse::<Uuid>()?,
                name: msg
                    .field("NAME")
                    .ok_or(anyhow!("{} field not found", "NAME".bold()))?
                    .into(),
            },
            "EXITED" => InvokerToManager::Exited {
                code: msg
                    .field("CODE")
                    .ok_or(anyhow!("{} filed not found", "CODE".bold()))?
                    .parse()
                    .context("parsing 'CODE' field")?,
                data: String::from_utf8(
                    msg.data()
                        .ok_or(anyhow!("{} not found", "data".bold()))?
                        .into(),
                )?
                .into(),
            },
            command => {
                bail!("incorrect command '{}'", command.bold());
            }
        })
    }
}
impl super::Outgo for InvokerToManager {
    fn into_raw(self) -> RawMessage {
        match self {
            Self::Token { token, name } => {
                let mut body = RawMessage::new("TOKEN");
                body.add_fields(vec![(&"ID", &token), (&"KEY", &name)]);
                body
            }
            Self::Exited { code, data } => {
                let mut body = RawMessage::new("EXITED");
                body.add_fields(vec![(&"CODE", &code), (&"MESSAGE", &data)]);
                body
            }
        }
    }
}
