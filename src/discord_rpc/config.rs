use super::DEFAULT_CLIENT_ID;
use dotenvy::dotenv;
use home::home_dir;
use inquire::Text;
use serde::{Deserialize, Serialize};
use std::{
    env::{var, VarError},
    fs::{create_dir_all, metadata, write},
    path::Path,
};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub api_key: String,
    pub client_id: String,
}

impl Config {
    #[allow(dead_code)]
    pub fn write(&self, file_path: &Path) {
        let dir = file_path.parent().expect("Could not get parent directory");
        let toml = toml::to_string(&self).expect("Could not convert config to TOML");

        if metadata(file_path).is_ok() {
            info!("Config file {} already exists", file_path.display());
            return;
        }

        if metadata(dir).is_ok() {
            info!("Writing to: {}", file_path.display());
            write(file_path, toml).expect("Could not write to config file");
        } else {
            info!("Creating config directory: {}", dir.display());
            info!("Writing to: {}", file_path.display());
            create_dir_all(dir).expect("Could not create config directory");
            write(file_path, toml).expect("Could not write to config file");
        }
    }
}

fn get_vars(key: &str) -> Result<String, VarError> {
    if var(key).is_err() {
        dotenv().ok();
        if var(key).is_err() {
            Err(VarError::NotPresent)
        } else {
            var(key)
        }
    } else {
        var(key)
    }
}

impl Default for Config {
    fn default() -> Self {
        let file_path = home_dir()
            .expect("Could not get home directory")
            .join(".config/lastfm-discord-rpc/config.toml");

        if Path::exists(&file_path) {
            let file = std::fs::read_to_string(file_path).expect("Could not read config file");
            let config: Config = toml::from_str(&file).expect("Could not parse config file");
            config
        } else {
            let (username, api_key, client_id) = (
                var("LASTFM_USERNAME"),
                var("LASTFM_API_KEY"),
                var("CLIENT_ID"),
            );

            if get_vars("LASTFM_USERNAME").is_err()
                || get_vars("LASTFM_API_KEY").is_err()
                || get_vars("CLIENT_ID").is_err()
            {
                let (username, api_key, client_id) = (
                    Text::new("What's your Last.fm username? : ")
                        .prompt()
                        .expect("Could not get username from prompt"),
                    Text::new("What's your Last.fm API key? : ")
                        .prompt()
                        .expect("Could not get API key from prompt"),
                    Text::new("Enter a discord developer client id (keep empty for default) : ")
                        .prompt()
                        .expect("Could not get client id from prompt"),
                );
                if client_id.is_empty() {
                    let config = Config {
                        username,
                        api_key,
                        client_id: DEFAULT_CLIENT_ID.to_string(),
                    };
                    config.write(&file_path);
                    config
                } else {
                    let config = Config {
                        username,
                        api_key,
                        client_id,
                    };
                    config.write(&file_path);
                    config
                }
            } else {
                let (username, api_key, client_id) = (
                    username.expect("Couldn't get username"),
                    api_key.expect("Couldn't get api_key"),
                    client_id.expect("Couldn't get client_id"),
                );
                Config {
                    username,
                    api_key,
                    client_id,
                }
            }
        }
    }
}
