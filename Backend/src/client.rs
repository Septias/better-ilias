use hyper::{body::HttpBody, client::HttpConnector, Body, Client, Method, Request};
use hyper_tls::HttpsConnector;

use log::{error, info};
use scraper::Html;
use std::{fmt::Display, path::PathBuf, sync::{Arc, Mutex}};
use tokio::{fs::{File, create_dir_all}, io::AsyncWriteExt};

use crate::tree::IlNode;

type ClientType = Arc<hyper::Client<HttpsConnector<HttpConnector>>>;
pub struct IliasClient {
    client: ClientType,
    token: Option<String>,
}

#[derive(Debug)]
enum ClientError {
    NoToken,
    NoPath,
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::NoToken => {
                write!(f, "Client has no token")
            }
            ClientError::NoPath => {
                write!(f, "Requested File didn't answer with content-type")
            }
        }
    }
}

impl std::error::Error for ClientError {}

impl IliasClient {
    pub async fn get_page(
        &self,
        uri: &str,
    ) -> Result<Html, Box<dyn std::error::Error + Send + Sync>> {
        let req = Request::builder()
            .method(Method::GET)
            .uri("https://ilias.uni-freiburg.de/".to_owned() + &uri)
            .header(
                "cookie",
                "PHPSESSID=".to_owned() + &self.token.as_ref().ok_or(ClientError::NoToken)?,
            )
            .body(Body::empty())
            .unwrap();

        let mut resp = self.client.request(req).await?;
        if resp.status() != hyper::StatusCode::OK {
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
    pub fn new() -> Self {
        let https = HttpsConnector::new();
        IliasClient {
            client: Arc::new(Client::builder().build::<_, hyper::Body>(https)),
            token: None,
        }
    }
    pub fn set_token(&mut self, token: &str) {
        self.token = Some(token.to_string());
    }
    pub async fn download_file(
        &self,
        file_node: Arc<Mutex<IlNode>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let req = {
            let node = file_node.lock().unwrap();
            info!("Downloading file {}", node.title);
            Request::builder()
                .method(Method::GET)
                .uri(&node.uri)
                .header(
                    "cookie",
                    "PHPSESSID=".to_owned() + &self.token.as_ref().ok_or(ClientError::NoToken)?,
                )
                .body(Body::empty())
                .unwrap()
        };
        let mut resp = self.client.request(req).await?;

        let path = {
            let mut node = file_node.lock().unwrap();
            let path = node.breed.get_path().unwrap();
            let extension = resp
                .headers()
                .get("content-type")
                .ok_or_else(|| ClientError::NoPath)?
                .to_str()?
                .split("/")
                .nth(1)
                .unwrap();

            path.set_extension::<&str>(extension.into());
            path.clone()
        };


        create_dir_all(path.parent().unwrap()).await.expect(&format!{"{:?}", path});
        let mut file = File::create(path).await?;
        while let Some(chunk) = resp.body_mut().data().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        Ok(())
    }
}
