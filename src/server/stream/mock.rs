use std::future::pending;

use super::Error;

pub struct Mock<O, F: Fn(O)> {
    logger: F,
    _marker: std::marker::PhantomData<O>,
}

impl<O, F: Fn(O)> Mock<O, F> {
    pub fn new(logger: F) -> Self {
        Self {
            logger,
            _marker: Default::default(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("mock stream error")]
pub struct MockStreamError;

impl<I, O, F: Fn(O)> super::Stream<I, O> for Mock<O, F> {
    type AliveError = MockStreamError;
    type BrokenError = MockStreamError;
    fn recv(
        &self,
    ) -> impl Future<Output = Result<I, Error<Self::AliveError, Self::BrokenError>>> + Send {
        pending()
    }

    fn send(&self, msg: O) -> impl Future<Output = Result<(), Self::BrokenError>> + Send {
        (self.logger)(msg);
        async move { Ok(()) }
    }
}
