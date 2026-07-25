use std::future::pending;

use super::stream::Stream;

pub struct Mock {
    name: &'static str,
}

impl<I, O> Stream<I, O> for Mock {
    fn recv(&self) -> impl Future<Output = anyhow::Result<I>> + Send {
        log::trace!("recv mock '{}'", self.name);
        pending()
    }

    fn send(&self, _: O) -> impl Future<Output = anyhow::Result<()>> + Send {
        log::trace!("sending into mock '{}'", self.name);
        async move { Ok(()) }
    }
}
