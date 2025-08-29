use async_trait::async_trait;
use nve_core::error::NveError;
use nve_core::ports::http::HttpClient;
use serde::de::DeserializeOwned;

pub struct ReqwestHttp(pub reqwest::Client);

impl Default for ReqwestHttp {
    fn default() -> Self {
        Self(reqwest::Client::new())
    }
}

#[async_trait]
impl HttpClient for ReqwestHttp {
    async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, NveError> {
        let res = self.0.get(url).send().await?.error_for_status()?;
        Ok(res.bytes().await?.to_vec())
    }

    async fn get_json<T: DeserializeOwned + Send>(&self, url: &str) -> Result<T, NveError> {
        let res = self.0.get(url).send().await?.error_for_status()?;
        Ok(res.json().await?)
    }
}
