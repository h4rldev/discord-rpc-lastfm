use colored::Colorize;
use config::Config;
use crossterm::{
    cursor::{Hide, MoveTo},
    style::Print,
    terminal::{size, Clear, ClearType, SetTitle},
    ExecutableCommand,
};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use futures::{stream::FuturesUnordered, StreamExt};
use home::home_dir;
use requests::CurrentTrack;
use reqwest::Client;
use std::io::{stdout, Stdout};
use tokio::{
    spawn,
    time::{sleep, Duration},
};
use tracing::{error, info, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::{format::FmtSpan, Subscriber};
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

pub const DEFAULT_CLIENT_ID: &str = "1161781788306317513";

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let log_dir = if cfg!(windows) {
        home_dir()
            .expect("Could not get home directory")
            .join(".logs\\discord-rpc-lastfm")
            .display()
            .to_string()
    } else {
        "/var/log/".to_string()
    };
    let file_appender =
        RollingFileAppender::new(Rotation::DAILY, log_dir, "discord-rpc-lastfm.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = Subscriber::builder()
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::CLOSE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");
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

fn print_newline(stdout: &mut Stdout, count: u16) -> Result<(), std::io::Error> {
    for _ in 0..count {
        stdout.execute(Print("\n"))?;
    }
    Ok(())
}

fn clear_screen(stdout: &mut Stdout) -> Result<(), std::io::Error> {
    stdout
        .execute(MoveTo(0, 0))?
        .execute(Clear(ClearType::FromCursorDown))?
        .execute(MoveTo(0, 0))?;
    Ok(())
}

async fn status_screen(presence: Presence) -> tokio::io::Result<()> {
    let mut stdout = stdout();
    stdout
        .execute(Hide)?
        .execute(SetTitle("discord-rpc-lastfm"))?;
    clear_screen(&mut stdout)?;

    let output = format!(
        "{}\n{}\n{}",
        presence.details.green(),
        presence.state.red(),
        presence.large_text.yellow(),
    );
    let (width, height) = size()?;
    let lines = output.matches('\n').count() + 1;
    let lines = lines as u16;
    let vertical_padding = if height > lines {
        (height - lines) / 2
    } else {
        0
    };
    print_newline(&mut stdout, vertical_padding)?;
    let output_lines: Vec<&str> = output.lines().collect();
    for line in output_lines {
        stdout.execute(Print(format!("{:^width$}\n", line, width = width as usize)))?;
    }
    print_newline(&mut stdout, vertical_padding)?;
    stdout.execute(MoveTo(0, 0))?;
    sleep(Duration::from_secs_f32(0.5)).await;
    Ok(())
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
                return Err(error);
            }
        }
    }
    Ok(())
}

async fn get_scrobble(client: Client, config: Config) -> tokio::io::Result<Option<Presence>> {
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
    sleep(Duration::from_secs_f32(0.5)).await;
    match current_track.recenttracks.track[0].attr {
        Some(_) => {
            let track = current_track.recenttracks.track[0].clone();
            let image = track.image.last().expect("Couldn't get last image").clone();
            if verify_urls(current_track).await.is_ok() {
                /* trunk-ignore(trufflehog/FastlyPersonalToken) */
                /* trunk-ignore(trufflehog/Lastfm) */
                if image.text == "https://lastfm.freetls.fastly.net/i/u/300x300/2a96cbd8b46e442fc41c2b86b821562f.png" {
                    let presence = Presence {
                        state: track.artist.text,
                        details: track.name,
                        large_image:
                            "https://cdn.signed.host/643beee7355f27cfb8628f80/1XP4viYdypmFHAUDNb.png"
                                .to_string(),
                        large_text: track.album.text,
                        small_image:
                            "https://www.last.fm/static/images/lastfm_avatar_twitter.52a5d69a85ac.png"
                                .to_string(),
                        small_text: "last.fm".to_string(),
                        buttons: vec![("View on last.fm".to_string(), track.url.clone()), (format!("{} on last.fm", config.username), format!("https://last.fm/user/{}", config.username))],
                    };
                    status_screen(presence.clone()).await?;
                    Ok(Some(presence))
                } else {
                    let presence = Presence {
                        state: track.artist.text,
                        details: track.name,
                        large_image: image.text,
                        large_text: track.album.text,
                        small_image:
                            "https://www.last.fm/static/images/lastfm_avatar_twitter.52a5d69a85ac.png"
                                .to_string(),
                        small_text: "last.fm".to_string(),
                        buttons: vec![
                            ("View on last.fm".to_string(), track.url.clone()),
                            (
                                format!("{} on last.fm", config.username),
                                format!("https://last.fm/user/{}", config.username),
                            ),
                        ],
                    };
                    status_screen(presence.clone()).await?;
                    Ok(Some(presence))
                }
            } else {
                let presence = Presence {
                    state: track.artist.text,
                    details: track.name,
                    large_image:
                        "https://cdn.signed.host/643beee7355f27cfb8628f80/1XP4viYdypmFHAUDNb.png"
                            .to_string(),
                    large_text: track.album.text,
                    small_image:
                        "https://www.last.fm/static/images/lastfm_avatar_twitter.52a5d69a85ac.png"
                            .to_string(),
                    small_text: "last.fm".to_string(),
                    buttons: vec![
                        ("View on last.fm".to_string(), track.url.clone()),
                        (
                            format!("{} on last.fm", config.username),
                            format!("https://last.fm/user/{}", config.username),
                        ),
                    ],
                };
                Ok(Some(presence))
            }
        }
        None => Ok(None),
    }
}
