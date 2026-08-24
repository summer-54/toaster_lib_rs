use super::{
    super::stream::master::{InvokerToManager, ManagerToInvoker},
    pb,
};
use crate::prelude::*;
impl TryInto<InvokerToManager> for pb::MasterIncome {
    type Error = Error;
    fn try_into(self) -> Result<InvokerToManager> {
        let Some(payload) = self.payload else {
            bail!("payload not found");
        };
        Ok(match payload {
            pb::master_income::Payload::Exited(exited) => InvokerToManager::Exited {
                code: exited.code as u8,
                data: exited.message.into(),
            },
        })
    }
}

impl From<InvokerToManager> for pb::MasterIncome {
    fn from(value: InvokerToManager) -> Self {
        Self {
            payload: Some(match value {
                InvokerToManager::Exited { code, data } => {
                    pb::master_income::Payload::Exited(pb::Exited {
                        code: code as u32,
                        message: data.into(),
                    })
                }
            }),
        }
    }
}

impl TryInto<ManagerToInvoker> for pb::MasterOutgo {
    type Error = Error;
    fn try_into(self) -> Result<ManagerToInvoker> {
        let Some(payload) = self.payload else {
            bail!("payload not found");
        };
        Ok(match payload {
            pb::master_outgo::Payload::Close(_) => ManagerToInvoker::Close,
        })
    }
}

impl From<ManagerToInvoker> for pb::MasterOutgo {
    fn from(value: ManagerToInvoker) -> Self {
        Self {
            payload: Some(match value {
                ManagerToInvoker::Close => pb::master_outgo::Payload::Close(pb::CloseInvoker {}),
            }),
        }
    }
}
