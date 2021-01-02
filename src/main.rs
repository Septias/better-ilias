use hyper::Client;
use hyper_tls::HttpsConnector;
use log::{error, info};
use std::{path::PathBuf, sync::Arc};
use sync::{FileSelect, FileWatcher, add_to_file_watcher};
use tree::{ get_or_create_ilias_tree};
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

    let mut  file_watcher = FileWatcher::new();

    let ilias_tree = get_or_create_ilias_tree(client.clone(), &mut file_watcher).await?;

    let ilias_tree = Box::leak(Box::new(ilias_tree));
    info!("sync ilias to local files");

    sync::sync(ilias_tree, client.clone()).await?;

    info!("sync files");
    add_to_file_watcher(ilias_tree, &mut file_watcher, "Bischte Dumm".to_string());  // remove pub decl
    //println!("files: {:#?}", file_watcher.files());
    file_watcher.sync(&mut*ilias_tree, FileSelect::All, client.clone()).await?;
    Ok(())
}
