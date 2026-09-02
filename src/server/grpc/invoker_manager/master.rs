use super::super::{
    super::stream::invoker_manager::master::{InvokerToManager, ManagerToInvoker},
    pb::invoker_manager as im_pb,
};
use crate::prelude::*;

impl From<InvokerToManager> for im_pb::MasterIncome {
    fn from(domain: InvokerToManager) -> Self {
        Self {
            payload: Some(match domain {
                InvokerToManager::Exited { code, data } => {
                    im_pb::master_income::Payload::Exited(im_pb::Exited {
                        code: code as u32,
                        message: data.into(),
                    })
                }
            }),
        }
    }
}

impl TryFrom<im_pb::MasterIncome> for InvokerToManager {
    type Error = Error;

    fn try_from(proto: im_pb::MasterIncome) -> Result<Self, Self::Error> {
        let Some(payload) = proto.payload else {
            bail!("payload not found");
        };

        Ok(match payload {
            im_pb::master_income::Payload::Exited(exited) => InvokerToManager::Exited {
                code: exited.code as u8,
                data: exited.message.into(),
            },
        })
    }
}

impl From<ManagerToInvoker> for im_pb::MasterOutgo {
    fn from(domain: ManagerToInvoker) -> Self {
        Self {
            payload: Some(match domain {
                ManagerToInvoker::Close => {
                    im_pb::master_outgo::Payload::Close(im_pb::CloseInvoker {})
                }
            }),
        }
    }
}

impl TryFrom<im_pb::MasterOutgo> for ManagerToInvoker {
    type Error = Error;

    fn try_from(proto: im_pb::MasterOutgo) -> Result<Self, Self::Error> {
        let Some(payload) = proto.payload else {
            bail!("payload not found");
        };

        Ok(match payload {
            im_pb::master_outgo::Payload::Close(_) => ManagerToInvoker::Close,
        })
    }
}
