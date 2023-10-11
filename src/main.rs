use config::{read_config, Config};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use dotenv::dotenv;
use request::CurrentTrack;
use reqwest::Client;
use std::env::current_dir;
use tracing::info;
mod config;
mod request;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv().expect("Couldn't find .env file");
    let request_client = Client::new();
    info!("Trying to write config file");
    read_config().await.expect("Couldn't find config");
    get_scrobble(
        request_client,
        read_config().await.expect("Couldn't find config"),
    )
    .await
    .expect("Couldn't get scrobble");
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
    std::thread::sleep(std::time::Duration::from_secs(15));
    Ok(())
}

async fn write_to_config(username: String, api_key: String) {
    let file_path = format!("{}/config/config.toml", current_dir().unwrap().display());
    let config = Config { username, api_key };
    config.write(&file_path).await;
}

async fn get_scrobble(
    client: Client,
    config: Config,
) -> Result<String, Box<dyn std::error::Error>> {
    info!("Getting scrobble for {}", config.username);
    let url = format!("http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user={}&api_key={}&format=json", config.username, config.api_key);
    let response = client.get(url).send().await?;
    let current_track = response.json::<CurrentTrack>().await?.recenttracks.track[0]
        .name
        .clone();
    info!("{:?}", current_track);
    Ok("yes".to_string())
}
