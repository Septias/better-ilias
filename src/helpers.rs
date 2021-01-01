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
use std::{
    collections::HashMap,
    io::ErrorKind,
    path::PathBuf,
    sync::{Arc, Mutex},
    unimplemented,
};
use tokio::{
    fs::{create_dir, File},
    io::{AsyncReadExt, AsyncWriteExt},
    task::{self, JoinHandle},
};

use config::Config;

use crate::config;

pub async fn request_il_page(
    uri: &str,
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
) -> Result<Html, Box<dyn std::error::Error + Send + Sync>> {
    let req = Request::builder()
        .method(Method::GET)
        .uri("https://ilias.uni-freiburg.de/".to_owned() + &uri)
        .header("cookie", "PHPSESSID=".to_owned() + Config::get_token())
        .body(Body::empty())
        .unwrap();
    let mut resp = client.request(req).await?;
    if resp.status() != StatusCode::OK {
        error!(
            "{} Problem with requestion ilias-page \" {}\"",
            resp.status(),
            uri
        );
    }
    let mut bytes = vec![];
    while let Some(chunk) = resp.body_mut().data().await {
        let chunk = chunk?;
        bytes.extend(&chunk[..]);
    }
    Ok(Html::parse_document(std::str::from_utf8(&bytes)?))
}
