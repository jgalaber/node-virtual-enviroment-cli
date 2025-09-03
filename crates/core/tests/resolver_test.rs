use nve_core::{
    domain::version::ParsedVersion, error::NveError, ports::http::HttpClient,
    services::ResolveService,
};

struct FakeHttp {
    payload: String,
}
#[async_trait::async_trait]
impl HttpClient for FakeHttp {
    async fn get_bytes(&self, _url: &str) -> Result<Vec<u8>, NveError> {
        Ok(self.payload.as_bytes().to_vec())
    }
    async fn get_json<T: serde::de::DeserializeOwned + Send>(
        &self,
        _url: &str,
    ) -> Result<T, NveError> {
        let v = serde_json::from_str::<T>(&self.payload)?;
        Ok(v)
    }
}

#[tokio::test]
async fn resolves_latest_matching_major() {
    let index = r#"
      [
        { "version":"v20.11.1", "date":"2024-01-01", "files": ["linux-x64"] },
        { "version":"v20.10.0", "date":"2023-12-01", "files": ["linux-x64"] },
        { "version":"v18.19.1", "date":"2023-10-01", "files": ["linux-x64"] }
      ]
    "#;

    let http = FakeHttp {
        payload: index.to_string(),
    };
    let svc = ResolveService { http: &http };
    let spec = ParsedVersion::parse("20").unwrap();

    let exact = svc.resolve(&spec).await.unwrap();
    assert_eq!(exact, "20.11.1");
}
