use serde::Deserialize;

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

pub struct Presence {
    pub state: String,
    pub details: String,
    pub large_image: String,
    pub large_text: String,
    pub small_image: String,
    pub small_text: String,
}
