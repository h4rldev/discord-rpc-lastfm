use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use config::{Config, read_config};
use reqwest::Client;
use request::CurrentTrack;
use std::env::current_dir;
use tracing_subscriber;
use tracing::info;
mod config;
mod request;


#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {

    tracing_subscriber::fmt::init();
    let request_client = Client::new();
    info!("Trying to write config file");
    write_to_config("1234".to_string(), "secret".to_string()).await;
    get_scrobble(request_client, read_config().await.expect("Couldn't find config")).await.expect("Couldn't get scrobble");
    info!("Trying to set RPC");
    test_rpc().await.expect("Couldn't set RPC");
}

async fn test_rpc() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DiscordIpcClient::new("1161781788306317513")?;
    client.connect()?;
    let activity = activity::Activity::new()
        .state("Testing")
        .details("Testing");
        /*.assets(
            activity::Assets::new()
            .large_image("large_image")
            .large_text("large_text")
            .small_image("small_image")
            .small_text("small_text"),
        );*/
    client.set_activity(activity)?;
    Ok(())
}

async fn write_to_config(username: String, api_key: String) {
    let file_path = format!("{}/config/config.toml", current_dir().unwrap().display());
    let config = Config {
        username,
        api_key,
    };
    config.write(&file_path).await;
}

async fn get_scrobble(client: Client, config: Config) -> Result<String, Box<dyn std::error::Error>> {
    info!("Getting scrobble for {}", config.username);
    let url = format!("http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user={}&api_key={}&format=json", config.username, config.api_key);
    let response = client.get(url).send().await?;
    let current_track = response.json::<CurrentTrack>().await?.recenttracks.track[0].name.clone();
    info!("{:?}", current_track);
    Ok("yes".to_string())
}