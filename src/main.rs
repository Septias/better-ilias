use hyper::Client;
use hyper_tls::HttpsConnector;
use std::{path::PathBuf, sync::Arc};
use sync::FileWatcher;
use tree::get_or_create_ilias_tree;

mod config;
mod helpers;
mod sync;
mod tree;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    let https = HttpsConnector::new();
    let client = Arc::new(Client::builder().build::<_, hyper::Body>(https));

    let file_watcher = Arc::new(FileWatcher::new());

    let ilias_tree = get_or_create_ilias_tree(client.clone(), file_watcher.clone()).await?;

    let ilias_tree = Box::leak(Box::new(ilias_tree));
    sync::sync(ilias_tree, PathBuf::new(), client.clone()).await?;

    Ok(())
}
