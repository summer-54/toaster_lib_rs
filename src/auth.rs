use std::{io::Write, ops::Deref, sync::Arc};

use rand::{CryptoRng, Rng};

use sequoia_openpgp::{
    self as pgp,
    parse::stream::{DetachedVerifierBuilder, MessageLayer, VerificationHelper},
    policy::Policy,
    serialize::stream::{Message, Signer},
};

pub use pgp::{Cert, parse::Parse, policy};

use serde::{Deserialize, Serialize};

struct Helper<'a>(&'a Cert);

impl<'a> VerificationHelper for Helper<'a> {
    fn get_certs(
        &mut self,
        _ids: &[sequoia_openpgp::KeyHandle],
    ) -> sequoia_openpgp::Result<Vec<Cert>> {
        Ok(vec![self.0.clone()])
    }

    fn check(
        &mut self,
        structure: sequoia_openpgp::parse::stream::MessageStructure<'_>,
    ) -> sequoia_openpgp::Result<()> {
        if let Some(error) = structure
            .into_iter()
            .find_map(|layer| -> Option<anyhow::Error> {
                let MessageLayer::SignatureGroup { results } = layer else {
                    return None;
                };
                results.into_iter().find_map(|result| {
                    let Err(error) = result else {
                        return None;
                    };
                    Some(anyhow::anyhow!("{error:?}"))
                })
            })
        {
            Err(error)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone)]
pub struct Challenge(Box<[u8]>);
pub struct Solution(Box<[u8]>);

impl From<&[u8]> for Challenge {
    fn from(bytes: &[u8]) -> Self {
        Self(Box::from(bytes))
    }
}

impl std::ops::Deref for Challenge {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&[u8]> for Solution {
    fn from(bytes: &[u8]) -> Self {
        Self(Box::from(bytes))
    }
}

impl std::ops::Deref for Solution {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SolvingError {
    #[error("valid pgp key not found")]
    ValidKeyNotFound,
    #[error("into keypair error: {0}")]
    IntoKeypair(anyhow::Error),
    #[error("signer creating error: {0}")]
    SignerCreating(anyhow::Error),
    #[error("message signing error: {0}")]
    MessageSigning(anyhow::Error),
    #[error("signer finalizing error: {0}")]
    SignerFinalizing(anyhow::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Challenge {
    pub fn generate<R: CryptoRng + Rng>(len: usize, rng: &mut R) -> Self {
        let mut bytes = vec![0u8; len].into_boxed_slice();
        rng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    pub fn solve(&self, cert: &Cert, policy: &impl Policy) -> Result<Solution, SolvingError> {
        let key = cert
            .keys()
            .supported()
            .secret()
            .with_policy(policy, None)
            .for_signing()
            .for_authentication()
            .next()
            .ok_or(SolvingError::ValidKeyNotFound)?
            .key()
            .clone()
            .into_keypair()
            .map_err(SolvingError::IntoKeypair)?;

        let mut bytes = Vec::<u8>::new();
        let mut sig = Signer::new(Message::new(&mut bytes), key)
            .map_err(SolvingError::SignerCreating)?
            .detached()
            .build()
            .map_err(SolvingError::MessageSigning)?;

        sig.write_all(&self.0)?;
        sig.finalize().map_err(SolvingError::SignerFinalizing)?;
        Ok(Solution(bytes.into()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("verifier builder creating error: {0}")]
    VerifierBuilderCreating(anyhow::Error),

    #[error("verifier building error: {0}")]
    VerifierBuilding(anyhow::Error),

    #[error("verification error: {0}")]
    Verification(anyhow::Error),
}

impl Solution {
    pub fn verify(
        &self,
        challenge: &Challenge,
        cert: &Cert,
        policy: &impl Policy,
    ) -> Result<(), VerificationError> {
        let verifier = DetachedVerifierBuilder::from_bytes(&self.0)
            .map_err(VerificationError::VerifierBuilding)?;
        let mut verifier = verifier
            .with_policy(policy, None, Helper(cert))
            .map_err(VerificationError::VerifierBuilding)?;
        verifier
            .verify_bytes(&**challenge)
            .map_err(VerificationError::Verification)?;
        Ok(())
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CertName(Arc<str>);

impl CertName {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> Arc<str> {
        self.0
    }

    pub fn new(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

impl Deref for CertName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<T: Into<Arc<str>>> From<T> for CertName {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Token(Arc<str>);

impl Token {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> Arc<str> {
        self.0
    }
}

impl Deref for Token {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<T: Into<Arc<str>>> From<T> for Token {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

#[test]
fn challendge_life_cycle() {
    use std::{io::Read, str::FromStr};
    let chg = Challenge::generate(10, &mut rand::rng());
    let bytes: Box<[u8]> = Box::from(&*chg);

    dbg!("send to invoker");

    let chg = Challenge::from(&*bytes);
    let cert = Cert::from_file("tests/sec.key").unwrap();
    let solution = chg.solve(&cert, &policy::StandardPolicy::new()).unwrap();
    let bytes: Box<[u8]> = Box::from(&*solution);

    dbg!("send to manager");

    let solution = Solution::from(&*bytes);
    let mut ct = String::new();
    std::fs::File::open("tests/pub.key")
        .unwrap()
        .read_to_string(&mut ct)
        .unwrap();
    let cert = Cert::from_str(&*ct).unwrap();
    solution
        .verify(&chg, &cert, &policy::StandardPolicy::new())
        .unwrap();
}
