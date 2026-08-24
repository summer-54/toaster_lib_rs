pub mod auth;
pub mod judge;
pub mod master;

use crate::prelude::*;

pub trait Stream<I, O> {
    fn recv(&self) -> impl Future<Output = Result<Result<I>>> + Send;
    fn send(&self, msg: O) -> impl Future<Output = Result<()>> + Send;
}
