use config::Config;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use requests::get_scrobble;
use reqwest::Client;
use tokio::spawn;
use tracing::{error, info, warn};

mod config;
mod log;
mod requests;
mod status_screen;

pub const DEFAULT_CLIENT_ID: &str = "1161781788306317513";

#[derive(Debug, Clone, PartialEq)]
pub struct Presence {
    pub state: String,
    pub details: String,
    pub large_image: String,
    pub large_text: String,
    pub small_image: String,
    pub small_text: String,
    pub buttons: Vec<(String, String)>,
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
            .buttons(vec![
                activity::Button::new(&self.buttons[0].0, &self.buttons[0].1),
                activity::Button::new(&self.buttons[1].0, &self.buttons[1].1),
            ])
    }
}

impl Default for Presence {
    fn default() -> Self {
        let lastfm_logo =
            "https://www.last.fm/static/images/lastfm_avatar_twitter.52a5d69a85ac.png".to_string();
        let missing_image =
            "https://cdn.signed.host/643beee7355f27cfb8628f80/1XP4viYdypmFHAUDNb.png".to_string();
        Self {
            state: "state".to_string(),
            details: "details".to_string(),
            large_image: missing_image,
            large_text: "large_text".to_string(),
            small_image: lastfm_logo,
            small_text: "last.fm".to_string(),
            buttons: vec![
                ("Button 1".to_string(), "https://example.com".to_string()),
                ("Button 2".to_string(), "https://example.com".to_string()),
            ],
        }
    }
}

pub async fn discord_rpc() {
    log::setup();
    info!("Trying to set RPC");
    let config = Config::default();
    let client_id = if config.client_id.is_empty() {
        DEFAULT_CLIENT_ID
    } else {
        &config.client_id
    };
    let mut rpc_client = DiscordIpcClient::new(client_id).expect("Could not set RPC");
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
        let scrobble = spawn(get_scrobble(Client::new(), Config::default()))
            .await
            .expect("Could not spawn get_scrobble")
            .expect("Could not get scrobble");

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
                            break;
                        }
                    }
                }
                None => {
                    let info = "No track playing".to_string();
                    if Some(info.clone()) != track_status {
                        track_status = Some(info.clone());
                        info!("{}", info);
                        loop {
                            if rpc_client.clear_activity().is_ok() {
                                warn!("Clearing activity");
                                break;
                            }
                        }
                    }
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
