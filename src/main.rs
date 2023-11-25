mod discord_rpc;

use discord_rpc::discord_rpc;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    discord_rpc().await;
}
