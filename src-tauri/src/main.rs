#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use ilias::IliasTree;
mod client;
mod ilias;
mod tree;

#[tokio::main]
async fn main() {
    env_logger::init();
    let tree = IliasTree::new().await;
    tree.run().await.unwrap();
}
