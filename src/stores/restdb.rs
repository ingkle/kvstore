use crate::store::KVStoreImpl;
use anyhow::Error;
use async_trait::async_trait;
use bytes::Bytes;
use hyper::StatusCode;
use url::Url;

#[derive(Clone)]
pub struct KVRestDB {
    url: Url,
}

impl KVRestDB {
    pub async fn try_new(url: Url) -> Result<Self, Error> {
        Ok(KVRestDB { url })
    }
}

#[async_trait]
impl KVStoreImpl for KVRestDB {
    async fn set(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let url = self
            .url
            .join(&format!("/keys/{}", std::str::from_utf8(key)?))?;
        reqwest::Client::new()
            .post(url)
            .body(String::from_utf8(value.to_vec())?)
            .send()
            .await?;

        Ok(())
    }

    async fn get(&self, key: &[u8]) -> Result<Option<Bytes>, Error> {
        let url = self
            .url
            .join(&format!("/keys/{}", std::str::from_utf8(key)?))?;
        let res = reqwest::get(url).await?;
        match res.status() {
            StatusCode::NOT_FOUND => Ok(None),
            _ => Ok(Some(res.bytes().await?)),
        }
    }

    async fn delete(&self, key: &[u8]) -> Result<(), Error> {
        let url = self
            .url
            .join(&format!("/keys/{}", std::str::from_utf8(key)?))?;
        reqwest::Client::new().delete(url).send().await?;

        Ok(())
    }

    async fn flush(&self) -> Result<(), Error> {
        Ok(())
    }
}
