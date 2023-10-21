use config::Config;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use futures::{stream::FuturesUnordered, StreamExt};
use requests::CurrentTrack;
use reqwest::Client;
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

impl Presence {
    fn convert_to_activity(&self) -> activity::Activity {
        activity::Activity::new()
            .state(&self.state)
            .details(&self.details)
            .assets(
                activity::Assets::new()
                    .large_image(&self.large_image)
                    .large_text(&self.large_text)
                    .small_image(&self.small_image)
                    .small_text(&self.small_text),
            )
    }
}

pub const DEFAULT_CLIENT_ID: &str = "1161781788306317513";

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Trying to set RPC");
    let config = Config::default();
    let client_id = if config.client_id.as_str().is_empty() {
        DEFAULT_CLIENT_ID
    } else {
        config.client_id.as_str()
    };
    let mut rpc_client = DiscordIpcClient::new(client_id).expect("Could not set RPC");
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
    let mut track_status: Option<String> = None;
    loop {
        let scrobble = spawn(get_scrobble(request_client.clone()))
            .await
            .expect("Could not spawn get_scrobble")
            .expect("Could not get scrobble");
        tokio::time::sleep(std::time::Duration::from_secs_f32(0.5)).await;

        if Some(scrobble.clone()) != last_scrobble {
            track_status = None;
            last_scrobble = Some(scrobble.clone());
            match scrobble {
                Some(presence) => {
                    info!("Track is now playing");
                    loop {
                        if rpc_client
                            .set_activity(presence.convert_to_activity())
                            .is_ok()
                        {
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
            let info = "No track playing".to_string();
            if Some(info.clone()) != track_status {
                track_status = Some(info.clone());
                info!("{}", info);
            }
        } else {
            let info = "Track is still playing".to_string();
            if Some(info.clone()) != track_status {
                track_status = Some(info.clone());
                info!("{}", info);
            }
        }
    }
}

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
    let current_track = loop {
        match CurrentTrack::new(client.clone()).await {
            Ok(current_track) => {
                break current_track;
            }
            Err(error) => {
                error!("Could not get current track: {}", error);
            }
        }
    };
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
