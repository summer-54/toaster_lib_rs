pub mod invoker_manager;
pub mod mock;
pub mod testing_system;
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error<A: std::error::Error, B: std::error::Error> {
    #[error("alive error: {0}")]
    Alive(A),
    #[error("broken error: {0}")]
    Broken(B),
}

pub trait Stream<I, O> {
    type AliveError: std::error::Error;
    type BrokenError: std::error::Error;
    fn recv(
        &self,
    ) -> impl Future<Output = Result<I, Error<Self::AliveError, Self::BrokenError>>> + Send;
    fn send(&self, msg: O) -> impl Future<Output = Result<(), Self::BrokenError>> + Send;
}
