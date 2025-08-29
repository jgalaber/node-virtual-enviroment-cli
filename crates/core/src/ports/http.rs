use crate::error::NveError;

#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, NveError>;
    async fn get_json<T: serde::de::DeserializeOwned + Send>(
        &self,
        url: &str,
    ) -> Result<T, NveError>;
}
