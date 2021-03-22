use hyper::{body::HttpBody, client::HttpConnector, Body, Client, Method, Request};
use hyper_tls::HttpsConnector;

use log::info;
use scraper::Html;
use std::{str::Utf8Error, sync::{Arc, Mutex, RwLock}};
use tokio::{fs::{File, create_dir_all}, io::AsyncWriteExt};
use thiserror::Error;
use crate::tree::IlNode;

type ClientType = Arc<hyper::Client<HttpsConnector<HttpConnector>>>;
pub struct IliasClient {
    client: ClientType,
    token: RwLock<Option<String>>,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Client has no token")]
    NoToken,
    #[error("Requested file didn't answer with content-type")]
    NoContentType,
    #[error("Client Error")]
    ClientError(#[from] hyper::Error),
    #[error("Parse Error")]
    ParesError(#[from] Utf8Error )
}


impl IliasClient {
    pub async fn get_page(
        &self,
        uri: &str,
    ) -> anyhow::Result<Html, ClientError> {
        let req = Request::builder()
            .method(Method::GET)
            .uri("https://ilias.uni-freiburg.de/".to_owned() + uri)
            .header(
                "cookie",
                "PHPSESSID=".to_owned() + &*self.token.read().unwrap().as_ref().ok_or(ClientError::NoToken)?,
            )
            .body(Body::empty())
            .unwrap();

        let mut resp = self.client.request(req).await?;
        if resp.status() != hyper::StatusCode::OK {
            return Err(ClientError::NoToken)
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
            token: RwLock::new(None),
        }
    }
    pub fn set_token(&self, token: &str) {
        let mut w = self.token.write().unwrap();
        *w = Some(token.to_string())
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
                    "PHPSESSID=".to_owned() + &*self.token.read().unwrap().as_ref().ok_or(ClientError::NoToken)?,
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
                .get("content-type").ok_or(ClientError::NoContentType)?
                .to_str()?
                .split('/')
                .nth(1)
                .unwrap();

            path.set_extension::<&str>(extension);
            path.clone()
        };
        create_dir_all(path.parent().unwrap()).await.unwrap_or_else(|_| panic!("{:?}", path));
        let mut file = File::create(path).await?;
        while let Some(chunk) = resp.body_mut().data().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        Ok(())
    }
}
