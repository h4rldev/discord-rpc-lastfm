use config::Config;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use futures::{stream::FuturesUnordered, StreamExt};
use requests::CurrentTrack;
use reqwest::Client;
use std::env::current_dir;
use tokio::spawn;
use tracing::{error, info, warn};
use url::{ParseError, Url};
mod config;
mod requests;

#[derive(Debug, Clone, PartialEq)]
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
    info!("Trying to set RPC");
    let mut rpc_client = DiscordIpcClient::new("1161781788306317513").expect("Could not set RPC");
    let request_client = Client::new();
    info!("Connecting to IPC");
    loop {
        if rpc_client.connect().is_ok() {
            info!("Connected to IPC");
            break;
        } else {
            error!("Could not connect to IPC, retrying");
        }
    }
    let mut last_scrobble: Option<Option<Presence>> = None;
    loop {
        let scrobble = spawn(get_scrobble(request_client.clone()))
            .await
            .expect("Could not spawn get_scrobble")
            .expect("Could not get scrobble");
        tokio::time::sleep(std::time::Duration::from_secs_f32(0.5)).await;

        if Some(scrobble.clone()) != last_scrobble {
            last_scrobble = Some(scrobble.clone());
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

                    info!("Track is now playing");
                    loop {
                        if rpc_client.set_activity(activity.clone()).is_ok() {
                            info!("Setting activity");
                            info!("Activity status:");
                            info!("State: {}", presence.state);
                            info!("Details: {}", presence.details);
                            info!("Large Image URL: {}", presence.large_image);
                            info!("Large Image Text: {}", presence.large_text);
                            info!("Small Image URL: {}", presence.small_image);
                            info!("Small Image Text: {}", presence.small_text);
                            break;
                        }
                    }
                    loop {
                        if rpc_client.set_activity(activity.clone()).is_ok() {
                            break;
                        }
                    }
                }
                None => {
                    warn!("No track playing");
                    loop {
                        if rpc_client.clear_activity().is_ok() {
                            warn!("Clearing activity");
                            break;
                        }
                    }
                    info!("Checking again in 10 seconds.");
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        } else if scrobble.is_none() {
            warn!("No track playing");
            info!("Checking again in 10 seconds.");
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        } else {
            info!("Track has not changed");
        }
    }
}

#[allow(dead_code)]
async fn write_to_config(username: String, api_key: String) {
    let file_path = format!("{}/config/config.toml", current_dir().unwrap().display());
    let config = Config { username, api_key };
    config.write(&file_path);
}

#[allow(dead_code)]
async fn verify_urls(track: CurrentTrack) -> Result<(), ParseError> {
    let image = track.recenttracks.track[0].image.clone();
    let mut is_url: FuturesUnordered<_> = image
        .into_iter()
        .map(|image| async move { Url::parse(&image.text) })
        .collect();

    if let Some(result) = is_url.next().await {
        match result {
            Ok(_) => {
                return Ok(());
            }
            Err(error) => {
                error!("Image is not a valid URL");
                return Err(error);
            }
        }
    }
    Ok(())
}

async fn get_scrobble(client: Client) -> tokio::io::Result<Option<Presence>> {
    let current_track = CurrentTrack::new(client)
        .await
        .expect("Can't get current track")
        .clone();
    match current_track.recenttracks.track[0].attr {
        Some(_) => {
            let track = current_track.recenttracks.track[0].clone();
            let image = track.image.last().unwrap().clone();
            if verify_urls(current_track).await.is_ok() {
                let presence = Presence {
                    state: track.artist.text,
                    details: track.name,
                    large_image: image.text,
                    large_text: track.album.text,
                    small_image:
                        "https://www.last.fm/static/images/lastfm_avatar_twitter.52a5d69a85ac.png"
                            .to_string(),
                    small_text: "last.fm".to_string(),
                };
                Ok(Some(presence))
            } else {
                let presence = Presence {
                    state: track.artist.text,
                    details: track.name,
                    large_image:
                        "https://media.discordapp.net/attachments/913382777993433149/1162078177636667496/questionmark.png"
                            .to_string(),
                    large_text: track.album.text,
                    small_image:
                        "https://www.last.fm/static/images/lastfm_avatar_twitter.52a5d69a85ac.png"
                            .to_string(),
                    small_text: "last.fm".to_string(),
                };
                Ok(Some(presence))
            }
        }
        None => Ok(None),
    }
}
