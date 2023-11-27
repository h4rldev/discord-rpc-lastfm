#![allow(dead_code)]

use super::{status_screen::status_screen, Config, Presence};
use futures::{stream::FuturesUnordered, StreamExt};
use reqwest::Client;
use serde::Deserialize;
use tokio::time::{sleep, Duration};
use tracing::error;
use url::{ParseError, Url};

#[derive(Deserialize, Debug, Clone)]
pub struct CurrentTrack {
    pub recenttracks: RecentTracks,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RecentTracks {
    pub track: Vec<Track>,
    #[serde(rename = "@attr")]
    pub attr: Attr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Track {
    pub artist: Artist,
    pub streamable: String,
    pub image: Vec<Image>,
    pub mbid: String,
    pub album: Album,
    pub name: String,
    #[serde(rename = "@attr")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attr: Option<TrackAttr>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<Date>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Artist {
    pub mbid: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Image {
    pub size: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Album {
    pub mbid: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Date {
    pub uts: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Attr {
    pub user: String,
    #[serde(rename = "totalPages")]
    pub totalpages: String,
    pub page: String,
    #[serde(rename = "perPage")]
    pub perpage: String,
    pub total: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TrackAttr {
    pub nowplaying: String,
}

impl CurrentTrack {
    pub async fn new(client: Client) -> Result<CurrentTrack, Box<dyn std::error::Error>> {
        let config = Config::default();
        let url = format!("http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user={}&api_key={}&format=json", config.username, config.api_key);
        Ok(client
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?
            .json::<CurrentTrack>()
            .await?)
    }
}

async fn verify_urls(track: &CurrentTrack) -> Result<(), ParseError> {
    let image = &track.recenttracks.track[0].image;
    let mut is_url: FuturesUnordered<_> = image
        .iter()
        .map(|image| async move { Url::parse(&image.text) })
        .collect();

    if let Some(result) = is_url.next().await {
        match result {
            Ok(_) => {}
            Err(error) => {
                return Err(error);
            }
        }
    }
    Ok(())
}

pub async fn get_scrobble(client: Client, config: Config) -> tokio::io::Result<Option<Presence>> {
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
            let track = current_track.recenttracks.track[0].to_owned();
            let image = track.image.last().expect("Couldn't get last image");
            let buttons = vec![
                ("View on last.fm".to_string(), track.url),
                (
                    format!("{} on last.fm", config.username),
                    format!("https://last.fm/user/{}", config.username),
                ),
            ];
            if verify_urls(&current_track).await.is_ok() {
                /* trunk-ignore(trufflehog/Lastfm) */
                /* trunk-ignore(trufflehog/FastlyPersonalToken) */
                if image.text == "https://lastfm.freetls.fastly.net/i/u/300x300/2a96cbd8b46e442fc41c2b86b821562f.png" {
                    let presence = Presence {
                        state: track.artist.text,
                        details: track.name,
                        large_text: track.album.text,
                        buttons,
                        ..Default::default()
                    };
                    status_screen(presence.clone()).await?;
                    Ok(Some(presence))
                } else {
                    let presence = Presence {
                        state: track.artist.text,
                        details: track.name,
                        large_image: image.text.clone(),
                        large_text: track.album.text,
                        buttons,
                        ..Default::default()
                    };
                    status_screen(presence.clone()).await?;
                    Ok(Some(presence))
                }
            } else {
                let presence = Presence {
                    state: track.artist.text,
                    details: track.name,
                    large_text: track.album.text,
                    buttons,
                    ..Default::default()
                };
                Ok(Some(presence))
            }
        }
        None => Ok(None),
    }
}
