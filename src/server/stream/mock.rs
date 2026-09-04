use crate::prelude::*;
use std::future::pending;

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

impl<I, O, F: Fn(O)> super::Stream<I, O> for Mock<O, F> {
    fn recv(&self) -> impl Future<Output = Result<Result<I>>> + Send {
        pending()
    }

    fn send(&self, msg: O) -> impl Future<Output = Result<()>> + Send {
        (self.logger)(msg);
        async move { Ok(()) }
    }
}
