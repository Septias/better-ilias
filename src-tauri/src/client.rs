use hyper::{body::HttpBody, client::HttpConnector, Body, Client, Method, Request};
use hyper_tls::HttpsConnector;

use crate::tree::IlNode;
use lazy_static::lazy_static;
use log::info;
use reqwest::{header::CONTENT_TYPE, redirect::Policy, ClientBuilder};
use scraper::{Html, Selector};
use std::{
    str::Utf8Error,
    sync::{Arc, Mutex, RwLock},
};
use thiserror::Error;
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
};

use urlencoding::encode;

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
    ParesError(#[from] Utf8Error),
    #[error("Reqwest Error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Shiat da scheinen die Logindaten nicht zu stimmen :/ uwu")]
    BadCredentials,
}

lazy_static! {
    pub static ref CONTEXT: Selector = Selector::parse("#LoginForm_context").unwrap();
    pub static ref INPUTS: Selector = Selector::parse("input").unwrap();
}

impl IliasClient {
    pub async fn get_page(&self, uri: &str) -> anyhow::Result<Html, ClientError> {
        let req = Request::builder()
            .method(Method::GET)
            .uri("https://ilias.uni-freiburg.de/".to_owned() + uri)
            .header(
                "cookie",
                "PHPSESSID=".to_owned()
                    + &*self
                        .token
                        .read()
                        .unwrap()
                        .as_ref()
                        .ok_or(ClientError::NoToken)?,
            )
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

    pub async fn acquire_token(&self, credentials: &[String; 2]) -> Result<String, ClientError> {
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
            .body(format!("LoginForm%5Bcontext%5D={}&LoginForm%5Busername%5D={}&LoginForm%5Bpassword%5D={}&yt0=Login", 
                encode(&context), credentials[0], credentials[1]))
            .header(CONTENT_TYPE,"application/x-www-form-urlencoded")
            .send()
            .await?.text().await?;

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
        self.set_token(&token);
        Ok(token)
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
                    "PHPSESSID=".to_owned()
                        + &*self
                            .token
                            .read()
                            .unwrap()
                            .as_ref()
                            .ok_or(ClientError::NoToken)?,
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
            .header(
                "cookie",
                "PHPSESSID=".to_owned()
                    + self
                    .token
                    .read()
                    .unwrap()
                    .as_ref()
                    .ok_or(ClientError::NoToken)?
            );

        let resp = preflight_req.send()
            .await?;

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
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use itertools::Itertools;
    use tokio::fs::read_to_string;

    use super::*;

    #[tokio::test]
    async fn flatten_link() {
        // this test is very specific to my setup/ilias-account so it will prob not work on yours or fail entirely after some time
        let node = Arc::new(Mutex::new(IlNode {
            id: 1,
            uri: "ilias.php?baseClass=ilLinkResourceHandlerGUI&ref_id=2070649&cmd=calldirectlink"
                .to_string(),
            title: "Ge".to_string(),
            breed: crate::tree::IlNodeType::DirectLink,
            parent: 12,
            visible: true,
            children: None,
        }));

        let client = IliasClient::new();
        if let Ok(raw_credentials) = read_to_string("credentials.txt").await {
            let credentials: [String; 2] = raw_credentials
                .split('\n')
                .map(|c| c.trim().to_owned())
                .collect_vec()
                .try_into()
                .unwrap();

            client.acquire_token(&credentials).await.unwrap();
            client.flatten_link(&node).await.unwrap();
            assert_eq!(
                &node.lock().unwrap().uri,
                &"https://uni-freiburg.zoom.us/j/62786894654"
            );
        }
    }
}
