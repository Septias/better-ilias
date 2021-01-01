use chrono::{DateTime, Utc};
use futures::future::join_all;
use hyper::{
    body::HttpBody as _, client::HttpConnector, Body, Client, Method, Request, StatusCode,
};
use hyper_tls::HttpsConnector;
use log::{error, info};
use ron::{
    de::from_bytes,
    from_str,
    ser::{to_string_pretty, PrettyConfig},
};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use sync::FileWatcher;
use tokio::{
    fs::{create_dir, File},
    io::{AsyncReadExt, AsyncWriteExt},
    task::{self, JoinHandle},
};
use tree::get_or_create_ilias_tree;

mod config;
mod helpers;
mod sync;
mod tree;

use tree::IlNode;

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
