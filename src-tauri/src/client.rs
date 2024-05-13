use crate::ilias::IlNode;
use crate::string_serializer;
use ::tauri::api::path::config_dir;
use anyhow::{anyhow, Context, Result};
use headless_chrome::Browser;
use lazy_static::lazy_static;
use log::{info, warn};
use reqwest::{redirect::Policy, Client, ClientBuilder, Method, StatusCode};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self},
    io,
    path::PathBuf,
    str::Utf8Error,
    sync::{Arc, Mutex},
};
use tauri::api::file::read_string;
use thiserror::Error;
use tokio::fs::create_dir_all;

#[derive(Debug, Error, Serialize)]
pub enum ClientError {
    #[error("Client has no token")]
    NoToken,
    #[error("Requested file didn't answer with content-type")]
    NoContentType,
    #[error("Parse Error")]
    #[serde(with = "string_serializer")]
    Parser(#[from] Utf8Error),
    #[error("Reqwest Error")]
    #[serde(with = "string_serializer")]
    Reqwest(#[from] reqwest::Error),
    #[error("Shiat da scheinen die Logindaten nicht zu stimmen :/ uwu")]
    BadCredentials,
    #[error(transparent)]
    #[serde(with = "string_serializer")]
    Anyhow(#[from] anyhow::Error),
}

lazy_static! {
    pub static ref CONTEXT: Selector = Selector::parse("#LoginForm_context").unwrap();
    pub static ref INPUTS: Selector = Selector::parse("input").unwrap();
}

pub struct IliasClient {
    token: String,
    client: Client,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Credentials {
    name: String,
    pw: String,
}

fn creds_path() -> Option<PathBuf> {
    config_dir().map(|mut path| {
        path.push("better-ilias/credentials.json");
        path
    })
}

fn load_creds() -> Result<Credentials> {
    let path = creds_path().ok_or(anyhow!("can't create path"))?;
    Ok(serde_json::from_str::<Credentials>(&read_string(path)?)?)
}

fn save_creds(creds: &Credentials) -> Result<()> {
    let path = creds_path().ok_or(anyhow!("can't create path"))?;
    if let Err(e) = fs::write(&path, serde_json::to_string(creds)?) {
        if e.kind() == io::ErrorKind::NotFound {
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(path, serde_json::to_string(creds)?)?;
        }
    }
    Ok(())
}

impl IliasClient {
    pub async fn new() -> Result<Self> {
        let creds = load_creds()?;
        let token = Self::acquire_token(&creds).await?;
        Ok(IliasClient {
            token,
            client: Client::new(),
        })
    }

    pub async fn with_creds(creds: Credentials) -> Result<Self, ClientError> {
        let token = Self::acquire_token(&creds).await?;
        Ok(IliasClient {
            token,
            client: Client::new(),
        })
    }

    pub async fn acquire_token(creds: &Credentials) -> Result<String, ClientError> {
        let browser = Browser::default()?;
        let tab = browser.new_tab()?;
        tab.navigate_to("https://ilias.uni-freiburg.de/shib_login.php?target=")?;

        tab.wait_for_element("input#LoginForm_username")?.click()?;
        tab.type_str(&creds.name)?;

        tab.wait_for_element("input#LoginForm_password")?.click()?;
        tab.type_str(&creds.pw)?.press_key("Enter")?;
        // This waits so long, maybe it is optimizable
        match tab.wait_for_element("#headerimage") {
            Ok(_) => {
                let token = tab
                    .get_cookies()?
                    .iter()
                    .find(|elem| elem.name == "PHPSESSID")
                    .ok_or(anyhow!("No cookie PHPSESSID"))?
                    .value
                    .clone();

                if let Err(err) = save_creds(creds) {
                    warn!("couldn't save credentials because of: {err}")
                }
                Ok(token)
            }
            Err(_) => Err(ClientError::BadCredentials),
        }
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

    pub async fn get_page(&self, uri: &str) -> Result<Html, ClientError> {
        let req = self
            .client
            .request(
                Method::GET,
                "https://ilias.uni-freiburg.de/".to_owned() + uri,
            )
            .header("cookie", "PHPSESSID=".to_owned() + &*self.token)
            .build()
            .context("can't build request")?;

        let resp = self.client.execute(req).await?;
        if resp.status() != StatusCode::OK {
            return Err(ClientError::NoToken);
        }
        Ok(Html::parse_document(&resp.text().await?))
    }

    pub async fn download_file(&self, file_node: Arc<Mutex<IlNode>>) -> Result<()> {
        let req = {
            let node = file_node.lock().unwrap();
            self.client
                .request(Method::GET, &node.uri)
                .header("cookie", "PHPSESSID=".to_owned() + &self.token)
                .build()?
        };
        let resp = self.client.execute(req).await?;

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

        if path.exists() {
            return Ok(());
        }

        let extension = path.extension().unwrap();
        if extension == "mp4" || extension == "zip" {
            *file_node.lock().unwrap().breed.get_local().unwrap() = false;
        }

        if *file_node.lock().unwrap().breed.get_local().unwrap() {
            create_dir_all(path.parent().unwrap())
                .await
                .unwrap_or_else(|_| panic!("{:?}", path));
            info!("Downloading file {:?}", path);
            fs::write(path, resp.text().await?)?;
        }

        Ok(())
    }
}
