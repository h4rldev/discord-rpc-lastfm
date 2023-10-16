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
            let (username, api_key) = (var("LASTFM_USERNAME"), var("LASTFM_API_KEY"));

            if var("LASTFM_USERNAME").is_err() || var("LASTFM_API_KEY").is_err() {
                dotenv().ok();
                let (username, api_key) = (var("LASTFM_USERNAME"), var("LASTFM_API_KEY"));

                if var("LASTFM_USERNAME").is_err() || var("LASTFM_API_KEY").is_err() {
                    let (username, api_key) = (
                        Text::new("What's your Last.fm username?")
                            .prompt()
                            .expect("Could not get username from prompt"),
                        Text::new("What's your Last.fm API key?")
                            .prompt()
                            .expect("Could not get API key from prompt"),
                    );
                    let config = Config { username, api_key };
                    config.write(&file_path);
                    config
                } else {
                    let (username, api_key) = (
                        username.expect("Couldn't get username"),
                        api_key.expect("Couldn't get api_key"),
                    );
                    Config { username, api_key }
                }
            } else {
                let (username, api_key) = (
                    username.expect("Couldn't get username"),
                    api_key.expect("Couldn't get api_key"),
                );
                Config { username, api_key }
            }
        }
    }
}
