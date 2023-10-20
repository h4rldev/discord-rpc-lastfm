use crate::DEFAULT_CLIENT_ID;
use dotenv::dotenv;
use inquire::Text;
use serde::{Deserialize, Serialize};
use std::{
    env::{current_dir, var},
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
    pub fn write(&self, file_path: &str) {
        let dir = Path::new(&file_path)
            .parent()
            .expect("Could not get parent directory");
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

impl Default for Config {
    fn default() -> Self {
        let file_path = format!("{}/config/config.toml", current_dir().unwrap().display());

        if metadata(&file_path).is_ok() {
            let file = std::fs::read_to_string(file_path).expect("Could not read config file");
            let config: Config = toml::from_str(&file).expect("Could not parse config file");
            config
        } else {
            let (username, api_key, client_id) = (
                var("LASTFM_USERNAME"),
                var("LASTFM_API_KEY"),
                var("CLIENT_ID"),
            );

            if var("LASTFM_USERNAME").is_err()
                || var("LASTFM_API_KEY").is_err()
                || var("CLIENT_ID").is_err()
            {
                dotenv().ok();
                let (username, api_key, client_id) = (
                    var("LASTFM_USERNAME"),
                    var("LASTFM_API_KEY"),
                    var("CLIENT_ID"),
                );

                if var("LASTFM_USERNAME").is_err()
                    || var("LASTFM_API_KEY").is_err()
                    || var("CLIENT_ID").is_err()
                {
                    let (username, api_key, client_id) = (
                        Text::new("What's your Last.fm username?")
                            .prompt()
                            .expect("Could not get username from prompt"),
                        Text::new("What's your Last.fm API key?")
                            .prompt()
                            .expect("Could not get API key from prompt"),
                        Text::new("Enter a discord developer client id, keep empty for default")
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
