use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CurrentTrack {
    pub recenttracks: RecentTracks,
}

#[derive(Deserialize, Debug)]
pub struct RecentTracks {
    pub track: Vec<Track>,
    #[serde(rename = "@attr")]
    pub attr: Attr,
}

#[derive(Deserialize, Debug)]
pub struct Track {
    pub artist: Artist,
    pub streamable: String,
    pub image: Vec<Image>,
    pub mbid: String,
    pub album: Album,
    pub name: String,
    #[serde(rename = "@attr")]
    pub attr: Option<TrackAttr>,
    pub url: String,
    pub date: Option<Date>,
}

#[derive(Deserialize, Debug)]
pub struct Artist {
    pub mbid: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct Image {
    pub size: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct Album {
    pub mbid: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct Date {
    pub uts: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct Attr {
    pub user: String,
    #[serde(rename = "totalPages")]
    pub totalpages: String,
    pub page: String,
    #[serde(rename = "perPage")]
    pub perpage: String,
    pub total: String,
}

#[derive(Deserialize, Debug)]
pub struct TrackAttr {
    pub nowplaying: String,
}