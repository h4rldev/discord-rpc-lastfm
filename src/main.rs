#![allow(unused_imports)]

mod discord_rpc;

use discord_rpc::discord_rpc;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    discord_rpc::gui::init();
    Ok(())
    //discord_rpc().await;
}
