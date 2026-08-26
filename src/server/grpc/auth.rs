use super::{super::stream::auth, pb};
use crate::{auth::Challenge, prelude::*};
impl TryInto<auth::InvokerToManager> for pb::AuthIncome {
    type Error = Error;
    fn try_into(self) -> Result<auth::InvokerToManager> {
        let Some(payload) = self.payload else {
            bail!("payload not found");
        };
        Ok(match payload {
            pb::auth_income::Payload::Cert(cert_name) => {
                auth::InvokerToManager::CertName(cert_name.name.into())
            }
            pb::auth_income::Payload::Proof(auth_proof) => {
                auth::InvokerToManager::AuthProof((&*auth_proof.data).into())
            }
        })
    }
}

impl From<auth::InvokerToManager> for pb::AuthIncome {
    fn from(value: auth::InvokerToManager) -> Self {
        Self {
            payload: Some(match value {
                auth::InvokerToManager::AuthProof(solution) => {
                    pb::auth_income::Payload::Proof(pb::AuthProof {
                        data: Vec::from(&*solution),
                    })
                }
                auth::InvokerToManager::CertName(name) => {
                    pb::auth_income::Payload::Cert(pb::CertName {
                        name: name.to_string(),
                    })
                }
            }),
        }
    }
}

impl TryInto<auth::ManagerToInvoker> for pb::AuthOutgo {
    type Error = Error;
    fn try_into(self) -> Result<auth::ManagerToInvoker> {
        let Some(payload) = self.payload else {
            bail!("payload not found");
        };
        Ok(match payload {
            pb::auth_outgo::Payload::Challenge(auth_challenge) => {
                auth::ManagerToInvoker::Challenge(Challenge::from(&*auth_challenge.data))
            }
            pb::auth_outgo::Payload::Verdict(verdict) => {
                auth::ManagerToInvoker::Verdict(match pb::AuthVerdict::try_from(verdict)? {
                    pb::AuthVerdict::AuthUnspecified => bail!("AuthVerdict unspecified"),
                    pb::AuthVerdict::Approved => true,
                    pb::AuthVerdict::Denied => false,
                })
            }
        })
    }
}

impl From<auth::ManagerToInvoker> for pb::AuthOutgo {
    fn from(value: auth::ManagerToInvoker) -> Self {
        Self {
            payload: Some(match value {
                auth::ManagerToInvoker::Challenge(challenge) => {
                    pb::auth_outgo::Payload::Challenge(pb::AuthChallenge {
                        data: Vec::from(&*challenge),
                    })
                }
                auth::ManagerToInvoker::Verdict(verdict) => {
                    pb::auth_outgo::Payload::Verdict(match verdict {
                        true => pb::AuthVerdict::Approved.into(),
                        false => pb::AuthVerdict::Denied.into(),
                    })
                }
            }),
        }
    }
}
