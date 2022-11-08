use hyper::{body::HttpBody, client::HttpConnector, Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
};

use crate::ilias::IlNode;
use lazy_static::lazy_static;
use reqwest::{header::CONTENT_TYPE, redirect::Policy, ClientBuilder};
use scraper::{Html, Selector};
use std::{
    fs::{self, read_to_string},
    str::Utf8Error,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use urlencoding::encode;

type ClientType = Arc<hyper::Client<HttpsConnector<HttpConnector>>>;
pub struct IliasClient {
    client: ClientType,
    token: String,
}
use crate::string_serializer;

#[derive(Debug, Error, Serialize)]
pub enum ClientError {
    #[error("Client has no token")]
    NoToken,
    #[error("Requested file didn't answer with content-type")]
    NoContentType,

    #[error("Client Error")]
    #[serde(with = "string_serializer")]
    Client(#[from] hyper::Error),
    #[error("Parse Error")]
    #[serde(with = "string_serializer")]
    Parser(#[from] Utf8Error),
    #[error("Reqwest Error")]
    #[serde(with = "string_serializer")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Shiat da scheinen die Logindaten nicht zu stimmen :/ uwu")]
    BadCredentials,
}

lazy_static! {
    pub static ref CONTEXT: Selector = Selector::parse("#LoginForm_context").unwrap();
    pub static ref INPUTS: Selector = Selector::parse("input").unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    name: String,
    pw: String,
}

fn load_creds() -> anyhow::Result<Credentials> {
    Ok(serde_json::from_str::<Credentials>(&read_to_string(
        "credentials.json",
    )?)?)
}

fn save_creds(creds: &Credentials) -> anyhow::Result<()> {
    fs::write("credentials.json", serde_json::to_string(creds)?)?;
    Ok(())
}

impl IliasClient {
    pub async fn get_page(&self, uri: &str) -> anyhow::Result<Html, ClientError> {
        let req = Request::builder()
            .method(Method::GET)
            .uri("https://ilias.uni-freiburg.de/".to_owned() + uri)
            .header("cookie", "PHPSESSID=".to_owned() + &*self.token)
            .body(Body::empty())
            .unwrap();

        let mut resp = self.client.request(req).await?;
        if resp.status() != hyper::StatusCode::OK {
            return Err(ClientError::NoToken);
        }
        let mut bytes = vec![];
        while let Some(chunk) = resp.body_mut().data().await {
            let chunk = chunk?;
            bytes.extend(&chunk[..]);
        }
        Ok(Html::parse_document(std::str::from_utf8(&bytes)?))
    }

    pub async fn with_creds(creds: Credentials) -> Result<Self, ClientError> {
        let client = Arc::new(Client::builder().build::<_, hyper::Body>(HttpsConnector::new()));
        let token = Self::acquire_token(&creds).await?;
        save_creds(&creds).map_err(|e| warn!("{e}")).ok();
        Ok(IliasClient { client, token })
    }

    pub async fn new() -> anyhow::Result<Self> {
        let client = Arc::new(Client::builder().build::<_, hyper::Body>(HttpsConnector::new()));
        let creds = load_creds()?;
        let token = Self::acquire_token(&creds).await?;
        Ok(IliasClient { client, token })
    }

    pub fn _new_with_token(token: String) -> Self {
        let https = HttpsConnector::new();
        IliasClient {
            client: Arc::new(Client::builder().build::<_, hyper::Body>(https)),
            token,
        }
    }

    pub async fn acquire_token(creds: &Credentials) -> Result<String, ClientError> {
        let client = ClientBuilder::new()
            .cookie_store(true)
            .http1_title_case_headers()
            .build()?;

        // request to get context and auth-url
        let resp = client
            .get("https://ilias.uni-freiburg.de/shib_login.php?target=")
            .send()
            .await?;

        let url = resp.url().as_str().to_owned();
        let resp_body = resp.text().await?;

        let context = {
            let document = Html::parse_document(&resp_body);
            document
                .select(&CONTEXT)
                .next()
                .expect("No Context found")
                .value()
                .attr("value")
                .expect("No Value Field")
                .to_owned()
        };

        // request relay_state and SAMLResponse
        let resp_body = client
            .post(url)
            .body(format!(
        "LoginForm%5Bcontext%5D={}&LoginForm%5Busername%5D={}&LoginForm%5Bpassword%5D={}&yt0=Login",
        encode(&context),
        creds.name,
        creds.name
      ))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .send()
            .await?
            .text()
            .await?;

        let (relay_state, samlresponse) = {
            let html = Html::parse_document(&resp_body);
            let mut inputs = html.select(&INPUTS);
            let relay_state = inputs.next().unwrap().value().attr("value").unwrap();
            let samlresponse = inputs.next().unwrap().value().attr("value").unwrap();
            (relay_state.to_owned(), samlresponse.to_owned())
        };

        let client = reqwest::Client::builder()
            .redirect(Policy::custom(|attempt| {
                if attempt.previous().len() > 1 {
                    attempt.stop()
                } else {
                    attempt.follow()
                }
            }))
            .cookie_store(true)
            .build()
            .unwrap();

        // make final call to ilias to acquire PHPSESSID
        let resp = client
            .post("https://ilias.uni-freiburg.de/Shibboleth.sso/SAML2/POST")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(format!(
                "RelayState={}&SAMLResponse={}",
                encode(&relay_state),
                encode(&samlresponse)
            ))
            .send()
            .await?;

        let sess_id = resp
            .cookies()
            .find(|c| c.name() == "PHPSESSID")
            .ok_or(ClientError::BadCredentials)?;

        let token = sess_id.value().to_string();
        if save_creds(creds).is_err() {
            warn!("couldn't save credenetials")
        }
        Ok(token)
    }

    /// Takes a ilias-link (which is a redirect) and replaces it with the correct location
    pub async fn flatten_link(&self, node: &Arc<Mutex<IlNode>>) -> Result<(), ClientError> {
        let client = ClientBuilder::new()
            .cookie_store(true)
            .http1_title_case_headers()
            .redirect(Policy::none())
            .build()?;

        // request to get context and auth-url
        let uri = node.lock().unwrap().uri.clone();

        let preflight_req = client
            .get("https://ilias.uni-freiburg.de/".to_string() + &uri)
            .header("cookie", "PHPSESSID=".to_owned() + &self.token);

        let resp = preflight_req.send().await?;

        let link_location = resp
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        node.lock().unwrap().uri = link_location;
        Ok(())
    }

    pub async fn download_file(&self, file_node: Arc<Mutex<IlNode>>) -> anyhow::Result<()> {
        let req = {
            let node = file_node.lock().unwrap();
            info!("Downloading file {}", node.title);
            Request::builder()
                .method(Method::GET)
                .uri(&node.uri)
                .header("cookie", "PHPSESSID=".to_owned() + &self.token)
                .body(Body::empty())
                .unwrap()
        };
        let mut resp = self.client.request(req).await?;

        let path = {
            let mut node = file_node.lock().unwrap();
            let path = node.breed.path_mut().unwrap();
            let extension = resp
                .headers()
                .get("content-type")
                .ok_or(ClientError::NoContentType)?
                .to_str()?
                .split('/')
                .nth(1)
                .unwrap();

            path.set_extension::<&str>(extension);
            path.clone()
        };

        if path.extension().unwrap() == "mp4" {
            *file_node.lock().unwrap().breed.get_local().unwrap() = false;
        }

        if *file_node.lock().unwrap().breed.get_local().unwrap() {
            create_dir_all(path.parent().unwrap())
                .await
                .unwrap_or_else(|_| panic!("{:?}", path));
            let mut file = File::create(path).await?;
            while let Some(chunk) = resp.body_mut().data().await {
                let chunk = chunk?;
                file.write_all(&chunk).await?;
            }
        }

        Ok(())
    }
}
