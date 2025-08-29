use crate::constants::NODEJS_API_INDEX;
use crate::domain::release::NodeRelease;
use crate::domain::version::{matches_semver, ParsedVersion};
use crate::error::NveError;
use crate::ports::http::HttpClient;

pub struct ResolveService<'a, H: HttpClient> {
    pub http: &'a H,
}

impl<'a, H: HttpClient> ResolveService<'a, H> {
    pub async fn resolve(&self, spec: &ParsedVersion) -> Result<String, NveError> {
        let releases: Vec<NodeRelease> = self.http.get_json(NODEJS_API_INDEX).await?;

        releases
            .iter()
            .filter_map(|r| r.version.strip_prefix('v').map(str::to_string))
            .find(|v| matches_semver(v, spec))
            .ok_or_else(|| NveError::VersionNotFound(spec.full_version.clone()))
    }
}
