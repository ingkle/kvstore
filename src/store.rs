use crate::stores::restdb::KVRestDB;
use crate::stores::slatedb::KVSlateDB;
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use bytes::Bytes;
use url::Url;

pub type KVStoreRef = Box<dyn KVStoreImpl>;

#[derive(Clone)]
pub struct KVStore {}

impl KVStore {
    pub async fn try_new(url: Option<String>) -> Result<Box<dyn KVStoreImpl>, Error> {
        let url = url.unwrap_or("/tmp/kvstore".into());
        let url = match Url::parse(&url) {
            Ok(url) => url,
            Err(url::ParseError::RelativeUrlWithoutBase) => {
                Url::from_file_path(&url).map_err(|_| anyhow!("could not parse relative path"))?
            }
            Err(err) => return Err(err.into()),
        };

        let scheme = url.scheme();
        let store: KVStoreRef = match scheme {
            "s3" | "file" => Box::new(KVSlateDB::try_new(url).await?),
            "http" | "https" => Box::new(KVRestDB::try_new(url).await?),
            _ => return Err(anyhow!("invalid store scheme")),
        };

        Ok(store)
    }
}

#[async_trait]
pub trait KVStoreImpl: Send + Sync {
    async fn set(&self, key: &[u8], value: &[u8]) -> Result<(), Error>;
    async fn get(&self, key: &[u8]) -> Result<Option<Bytes>, Error>;
    async fn delete(&self, key: &[u8]) -> Result<(), Error>;
    async fn flush(&self) -> Result<(), Error>;
}
