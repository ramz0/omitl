use super::BrandConfig;

pub fn load_brand(path: Option<&str>) -> anyhow::Result<BrandConfig> {
    match path {
        Some(p) => {
            let content = std::fs::read_to_string(p)?;
            let config: BrandConfig = serde_json::from_str(&content)
                .or_else(|_| serde_yaml::from_str(&content))?;
            Ok(config)
        }
        None => Ok(BrandConfig::default()),
    }
}
