use iori::cache::opendal::services::S3Config;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "s3-rec")]
    pub s3: S3Config,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let file = "config.toml";
        let data = std::fs::read_to_string(file)?;
        let config = toml::from_str(&data)?;
        Ok(config)
    }
}
