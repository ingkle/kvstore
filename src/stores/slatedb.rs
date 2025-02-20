use crate::store::KVStoreImpl;
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use bytes::Bytes;
use object_store::aws::{AmazonS3Builder, S3ConditionalPut};
use object_store::local::LocalFileSystem;
use object_store::path::Path;
use object_store::{ClientOptions, ObjectStore};
use slatedb::config::DbOptions;
use slatedb::db::Db;
use std::sync::Arc;
use url::Url;

#[derive(Clone)]
pub struct KVSlateDB {
    db: Arc<Db>,
}

impl KVSlateDB {
    pub async fn try_new(url: Url) -> Result<Self, Error> {
        let scheme = url.scheme();
        let store: Arc<dyn ObjectStore> = match scheme {
            "s3" => {
                let bucket = url.host_str().expect("could not get bucket name");
                let options = ClientOptions::new().with_timeout_disabled();
                let s3 = AmazonS3Builder::from_env()
                    .with_bucket_name(bucket)
                    .with_client_options(options)
                    .with_conditional_put(S3ConditionalPut::ETagMatch)
                    .with_allow_http(true)
                    .build()?;
                Arc::new(s3)
            }
            "file" => Arc::new(LocalFileSystem::default()),
            _ => return Err(anyhow!("invalid object store")),
        };
        let path = Path::from(url.path());

        let options = DbOptions::default();
        let db = Db::open_with_opts(path, options, store).await?;

        Ok(KVSlateDB { db: Arc::new(db) })
    }
}

#[async_trait]
impl KVStoreImpl for KVSlateDB {
    async fn set(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        self.db.put(key, value).await?;

        Ok(())
    }

    async fn get(&self, key: &[u8]) -> Result<Option<Bytes>, Error> {
        let value = self.db.get(key).await?;

        Ok(value)
    }

    async fn delete(&self, key: &[u8]) -> Result<(), Error> {
        self.db.delete(key).await?;

        Ok(())
    }

    async fn flush(&self) -> Result<(), Error> {
        self.db.flush().await?;

        Ok(())
    }
}
