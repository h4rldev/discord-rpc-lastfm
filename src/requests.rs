use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;

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
