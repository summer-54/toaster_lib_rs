use super::super::{super::stream::invoker_manager::auth, pb::invoker_manager as im_pb};
use crate::{auth::Challenge, prelude::*};

impl TryFrom<im_pb::AuthIncome> for auth::InvokerToManager {
    type Error = Error;

    fn try_from(proto: im_pb::AuthIncome) -> Result<Self> {
        let Some(payload) = proto.payload else {
            bail!("payload not found");
        };

        Ok(match payload {
            im_pb::auth_income::Payload::Proof(auth_proof) => {
                auth::InvokerToManager::AuthProof((&*auth_proof.data).into())
            }
        })
    }
}

impl From<auth::InvokerToManager> for im_pb::AuthIncome {
    fn from(msg: auth::InvokerToManager) -> Self {
        Self {
            payload: Some(match msg {
                auth::InvokerToManager::AuthProof(solution) => {
                    im_pb::auth_income::Payload::Proof(im_pb::AuthProof {
                        data: Vec::from(&*solution),
                    })
                }
            }),
        }
    }
}

impl TryFrom<im_pb::AuthOutgo> for auth::ManagerToInvoker {
    type Error = Error;

    fn try_from(proto: im_pb::AuthOutgo) -> Result<Self> {
        let Some(payload) = proto.payload else {
            bail!("payload not found");
        };

        Ok(match payload {
            im_pb::auth_outgo::Payload::Challenge(auth_challenge) => {
                auth::ManagerToInvoker::Challenge(Challenge::from(&*auth_challenge.data))
            }
            im_pb::auth_outgo::Payload::Verdict(verdict) => {
                auth::ManagerToInvoker::Verdict(match im_pb::AuthVerdict::try_from(verdict)? {
                    im_pb::AuthVerdict::Unspecified => bail!("AuthVerdict unspecified"),
                    im_pb::AuthVerdict::Approved => true,
                    im_pb::AuthVerdict::Denied => false,
                })
            }
        })
    }
}

impl From<auth::ManagerToInvoker> for im_pb::AuthOutgo {
    fn from(msg: auth::ManagerToInvoker) -> Self {
        Self {
            payload: Some(match msg {
                auth::ManagerToInvoker::Challenge(challenge) => {
                    im_pb::auth_outgo::Payload::Challenge(im_pb::AuthChallenge {
                        data: Vec::from(&*challenge),
                    })
                }
                auth::ManagerToInvoker::Verdict(verdict) => {
                    im_pb::auth_outgo::Payload::Verdict(match verdict {
                        true => im_pb::AuthVerdict::Approved.into(),
                        false => im_pb::AuthVerdict::Denied.into(),
                    })
                }
            }),
        }
    }
}
