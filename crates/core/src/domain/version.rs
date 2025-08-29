use semver::Version;

#[derive(Debug, Clone)]
pub struct ParsedVersion {
    pub major: u64,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub full_version: String,
}

impl ParsedVersion {
    pub fn parse(input: &str) -> Result<Self, String> {
        let parts: Vec<&str> = input.split('.').collect();
        if parts.is_empty() || parts[0].is_empty() {
            return Err(format!("Formato inválido: '{input}'"));
        }
        let major = parts[0].parse::<u64>().map_err(|_| format!("Formato inválido: '{input}'"))?;
        let minor = if parts.len() > 1 {
            Some(parts[1].parse::<u64>().map_err(|_| format!("Formato inválido: '{input}'"))?)
        } else { None };
        let patch = if parts.len() > 2 {
            Some(parts[2].parse::<u64>().map_err(|_| format!("Formato inválido: '{input}'"))?)
        } else { None };
        if parts.len() > 3 {
            return Err(format!("Formato inválido: '{input}'"));
        }
        Ok(Self { major, minor, patch, full_version: input.to_string() })
    }
}

pub fn matches_semver(ver: &str, spec: &ParsedVersion) -> bool {
    if let Ok(v) = Version::parse(ver) {
        if v.major != spec.major { return false; }
        if let Some(mn) = spec.minor { if v.minor != mn { return false; } }
        if let Some(p)  = spec.patch { if v.patch != p { return false; } }
        return true;
    }
    false
}
