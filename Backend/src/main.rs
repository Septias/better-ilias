use hyper::Client;
use hyper_tls::HttpsConnector;
use log::info;
use std::sync::Arc;
use sync::{FileSelect, FileWatcher};
use tree::get_or_create_ilias_tree;
mod config;
mod helpers;
mod sync;
mod tree;

pub type IdSize = u16;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    let https = HttpsConnector::new();
    let client = Arc::new(Client::builder().build::<_, hyper::Body>(https));

    let mut file_watcher = FileWatcher::new();

    let ilias_tree = get_or_create_ilias_tree(client.clone(), &mut file_watcher).await?;

    info!("sync structure to local filessystem");

    sync::sync(ilias_tree.clone(), client.clone()).await?;

    info!("sync files");
    //add_to_file_watcher(&ilias_tree.lock().unwrap(), &mut file_watcher, "Bischte Dumm".to_string()); //remove
    file_watcher
        .sync(ilias_tree, FileSelect::All, client.clone())
        .await?;
    Ok(())
}
