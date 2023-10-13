use config::{read_config, Config};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use futures::{stream::FuturesUnordered, StreamExt};
use reqwest::Client;
use std::env::current_dir;
use requests::{get_track, CurrentTrack};
use tracing::{info, error};
use url::{Url, ParseError};
use tokio::spawn;
mod config;
mod requests;

pub struct Presence {
    pub state: String,
    pub details: String,
    pub large_image: String,
    pub large_text: String,
    pub small_image: String,
    pub small_text: String,
}

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
    let config_task = spawn(read_config());
    let config = config_task.await?.expect("Could not read config file");
    let scrobble = spawn(get_scrobble(request_client, config)).await?.expect("Could not get scrobble");
    match scrobble {
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

#[allow(dead_code)]
async fn verify_urls(track: CurrentTrack) -> Result<(), ParseError> {
    let image = track.recenttracks.track[0].image.clone();
    let mut is_url: FuturesUnordered<_> = image.into_iter().map(|image| async move {
        return Url::parse(&image.text);
    }).collect();

    while let Some(result) = is_url.next().await {
        match result {
            Ok(_) => {
                info!("Image is a valid URL");
                return Ok(());
            }
            Err(_) => {
                error!("Image is not a valid URL");
                return Err(ParseError::RelativeUrlWithoutBase);
            }
        }
    }
    Ok(())
}


async fn get_scrobble(
    client: Client,
    config: Config,
) -> tokio::io::Result<Option<Presence>> {
    info!("Getting scrobble for {}", config.username);
    let current_track = get_track(client, config).await.expect("Can't get current track").clone();
    match current_track.recenttracks.track[0].attr {
        Some(_) => {
            info!("Track is now playing");
            let track = current_track.recenttracks.track[0].clone();
            let image = track.image.last().unwrap().clone();
            if verify_urls(current_track).await.is_ok() {
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
            Ok(None)
        }
    }
}
