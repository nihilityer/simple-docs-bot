use serde::{Deserialize, Serialize};
use crate::log::LogConfig;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use figment::providers::{Format, Json, Serialized, Toml, Yaml};
use figment::Figment;
use onebot_v11::connect::ws::WsConfig;

const JSON_CONFIG_FILE_NAME: &str = "config.json";
const TOML_CONFIG_FILE_NAME: &str = "config.toml";
const YAML_CONFIG_FILE_NAME: &str = "config.yaml";

#[derive(Deserialize, Serialize, Debug)]
pub struct CoreConfig {
    pub log: Vec<LogConfig>,
    pub data_base: DataBaseConfig,
    pub bot_ws: WsConfig,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DataBaseConfig{
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u64,
    pub database: String,
}

impl Default for CoreConfig {
    fn default() -> Self {
        CoreConfig {
            log: vec![LogConfig::default()],
            data_base: DataBaseConfig::default(),
            bot_ws: WsConfig::default(),
        }
    }
}

impl Default for DataBaseConfig {
    fn default() -> Self {
        DataBaseConfig{
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            host: "127.0.0.1".to_string(),
            port: 5432,
            database: "postgres".to_string(),
        }
    }
}

impl CoreConfig {
    pub fn init() -> Result<Self> {
        let config = CoreConfig::default();
        if Path::try_exists(TOML_CONFIG_FILE_NAME.as_ref())? {
            Ok(Figment::merge(
                Figment::from(Serialized::defaults(config)),
                Toml::file(TOML_CONFIG_FILE_NAME),
            )
                .extract()?)
        } else if Path::try_exists(YAML_CONFIG_FILE_NAME.as_ref())? {
            Ok(Figment::from(Serialized::defaults(config))
                .merge(Yaml::file(YAML_CONFIG_FILE_NAME))
                .extract()?)
        } else if Path::try_exists(JSON_CONFIG_FILE_NAME.as_ref())? {
            Ok(Figment::from(Serialized::defaults(config))
                .merge(Json::file(JSON_CONFIG_FILE_NAME))
                .extract()?)
        } else {
            let mut config_file: File = File::create(JSON_CONFIG_FILE_NAME)?;
            config_file.write_all(serde_json::to_string_pretty(&config)?.as_bytes())?;
            config_file.flush()?;
            Ok(config)
        }
    }
}