use colored::Colorize as _;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("file system error: {0}")]
    FileSystem(std::io::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_yml::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub trait Config: Default + Serialize + for<'de> Deserialize<'de>
where
    Self: 'static,
{
    const NAME: &'static str;
    fn load(dir: impl AsRef<std::path::Path>) -> impl Future<Output = Result<Self>> {
        async move {
            let path = dir.as_ref().join(Self::NAME).with_extension("yaml");
            if !tokio::fs::try_exists(&*path)
                .await
                .map_err(Error::FileSystem)?
            {
                let this = Self::default();
                let mut file = tokio::fs::File::create(&*path)
                    .await
                    .map_err(Error::FileSystem)?;
                file.write_all(serde_yml::to_string(&this)?.as_bytes())
                    .await
                    .map_err(Error::FileSystem)?;
                log::warn!(
                    "'{}' config not found by path: {}",
                    Self::NAME.bold().cyan(),
                    path.display()
                );
                log::debug!(
                    "'{}' config was automaticly created by path: {}",
                    Self::NAME.bold().cyan(),
                    path.display()
                );

                Ok(this)
            } else {
                let this = serde_yml::from_str(
                    &tokio::fs::read_to_string(&*path)
                        .await
                        .map_err(Error::FileSystem)?,
                )?;
                log::trace!("'{}' config was loaded", Self::NAME.bold().cyan());
                Ok(this)
            }
        }
    }
}
