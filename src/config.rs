use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    env::{var, current_dir},
    fs::{write, metadata, create_dir_all,},
};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub api_key: String,
}

impl Config {
    pub async fn write(&self, file_path: &str) {
        let dir = Path::new(&file_path).parent().expect("Could not get parent directory");
        let file = Path::new(&file_path);
        let toml = toml::to_string(&self).expect("Could not convert config to TOML");

        if metadata(file).is_ok() {
            info!("Config file {} already exists", file.display());
            return;
        }

        if metadata(dir).is_ok() {
            info!("Writing to: {}", file.display());
            write(file, toml).expect("Could not write to config file");
        } else {
            info!("Creating config directory: {}", dir.display());
            info!("Writing to: {}", file.display());
            create_dir_all(dir).expect("Could not create config directory");
            write(file, toml).expect("Could not write to config file");
        }
    }
}

pub async fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let file_path = format!("{}/config/config.toml", current_dir().unwrap().display());
    if metadata(&file_path).is_ok() {
        let file = std::fs::read_to_string(file_path)?;
        let config: Config = toml::from_str(&file)?;
        Ok(config)
    } else {
        let username = var("LASTFM_USERNAME").expect("Could not get username from environment variable");
        let api_key = var("LASTFM_API_KEY").expect("Could not get API key from environment variable");
        let config = Config {
            username,
            api_key,
        };
        Ok(config)
    }
}