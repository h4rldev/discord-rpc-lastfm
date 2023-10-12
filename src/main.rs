use config::{read_config, Config};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use reqwest::Client;
use std::env::current_dir;
use structs::{CurrentTrack, Presence};
use tracing::info;
use url::Url;
mod config;
mod structs;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Trying to write config file");
    info!("Trying to set RPC");
    test_rpc().await.expect("Couldn't set RPC");
}

async fn test_rpc() -> Result<(), Box<dyn std::error::Error>> {
    let mut rpc_client = DiscordIpcClient::new("1161781788306317513")?;
    let request_client = Client::new();
    let config = read_config().await?;
    match get_scrobble(request_client, config).await? {
        Some(presence) => {
            let activity = activity::Activity::new()
                .state(&presence.state)
                .details(&presence.details)
                .assets(
                    activity::Assets::new()
                        .large_image(&presence.large_image)
                        .large_text(&presence.large_text)
                        .small_image(&presence.small_image)
                        .small_text(&presence.small_text),
                );

            info!("Connecting to IPC");
            loop {
                if rpc_client.connect().is_ok() {
                    info!("Connected to IPC");
                    break;
                }
            }

            loop {
                if rpc_client.set_activity(activity.clone()).is_ok() {
                    info!("Set activity");
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(60));

            Ok(())
        }
        None => {
            info!("No track playing");
            Ok(())
        }
    }
}

#[allow(dead_code)]
async fn write_to_config(username: String, api_key: String) {
    let file_path = format!("{}/config/config.toml", current_dir().unwrap().display());
    let config = Config { username, api_key };
    config.write(&file_path).await;
}

async fn get_scrobble(
    client: Client,
    config: Config,
) -> Result<Option<Presence>, Box<dyn std::error::Error>> {
    info!("Getting scrobble for {}", config.username);
    let url = format!("http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user={}&api_key={}&format=json", config.username, config.api_key);
    let response = client.get(url).send().await?.json::<CurrentTrack>().await?;
    let current_track = response.clone();

    match current_track.recenttracks.track[0].attr {
        Some(_) => {
            info!("Track is now playing");
            let track = current_track.recenttracks.track[0].clone();
            let image = track.image.last().unwrap().clone();
            if Url::parse(&image.text).is_ok() {
                info!("Image is a valid URL");
                let presence = Presence {
                    state: track.artist.text,
                    details: format!("{} img size: {}", track.name, image.size),
                    large_image: image.text,
                    large_text: track.album.text,
                    small_image:
                        "https://www.last.fm/static/images/lastfm_avatar_twitter.52a5d69a85ac.png"
                            .to_string(),
                    small_text: "last.fm".to_string(),
                };
                info!(
                    "{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}",
                    presence.state,
                    presence.details,
                    presence.large_image,
                    presence.large_text,
                    presence.small_image,
                    presence.small_text
                );
                Ok(Some(presence))
            } else {
                info!("Image is not a valid URL");
                let presence = Presence {
                    state: track.artist.text,
                    details: format!("{} img size: {}", track.name, image.size),
                    large_image:
                        "https://media.discordapp.net/attachments/913382777993433149/1162078177636667496/questionmark.png"
                            .to_string(),
                    large_text: track.album.text,
                    small_image:
                        "https://www.last.fm/static/images/lastfm_avatar_twitter.52a5d69a85ac.png"
                            .to_string(),
                    small_text: "last.fm".to_string(),
                };
                info!(
                    "{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}",
                    presence.state,
                    presence.details,
                    presence.large_image,
                    presence.large_text,
                    presence.small_image,
                    presence.small_text
                );
                Ok(Some(presence))
            }
        }
        None => {
            info!("No track playing");
            Ok(None)
        }
    }
}
