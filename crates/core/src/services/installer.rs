use crate::domain::version::ParsedVersion;
use crate::error::NveError;
use crate::ports::archive::Archive;
use crate::ports::fs::FileSystem;
use crate::ports::http::HttpClient;
use crate::ports::platform::Platform;
use crate::services::ResolveService;
use crate::state::layout::NveLayout;

use crate::constants::NODEJS_API_BASE;

pub struct InstallService<'a, H, F, P, A> {
    pub http: &'a H,
    pub fs: &'a F,
    pub plat: &'a P,
    pub arch: &'a A,
    pub layout: &'a NveLayout,
}
impl<'a, H, F, P, A> InstallService<'a, H, F, P, A>
where
    H: HttpClient,
    F: FileSystem,
    P: Platform,
    A: Archive,
{
    pub async fn install(&self, spec: &ParsedVersion) -> Result<String, NveError> {
        let exact = ResolveService { http: self.http }.resolve(spec).await?;
        let version_dir = self.layout.version_dir(&exact);
        if self.fs.exists(&version_dir) {
            return Ok(exact);
        }

        let url = {
            let name = self.plat.archive_name(&exact);
            format!("{}/v{}/{}", NODEJS_API_BASE, &exact, name)
        };

        let data = self.http.get_bytes(&url).await?;
        self.fs.create_dir_all(&version_dir)?;
        self.arch.extract(&data, &version_dir, &exact).await?;
        Ok(exact)
    }
}
