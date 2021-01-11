use crate::{
    config::Config,
    helpers::{get_node, request_il_page},
    tree::{get_il_node_type, IlNode, IlNodeType, CONTAINERS, LINK, PROPERTY},
    IdSize,
};
use futures::future::join_all;
use hyper::{body::HttpBody, client::HttpConnector, Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use log::{error, info, warn};
use ron::{de::from_bytes, to_string};
use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::ErrorKind,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{
    fs::{create_dir, File},
    io::{AsyncReadExt, AsyncWriteExt},
    task::{self, JoinHandle},
};

pub fn sync(
    node: Arc<Mutex<IlNode>>,
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut sync_handles = vec![];
        let mut file_handles = vec![];
        {
            let node = node.lock().unwrap();
            match &node.breed {
                IlNodeType::Folder => {
                    if let Some(sync_info) = &node.sync {
                        file_handles.push(create_dir(sync_info.path.clone()));
                        if let Some(children) = &node.children {
                            for child in children
                                .iter()
                                .filter(|child| child.lock().unwrap().breed == IlNodeType::Folder)
                            {
                                sync_handles.push(sync(child.clone(), client.clone()));
                            }
                        }
                    }
                }
                IlNodeType::File => {}
                _ => (),
            };
        }

        for file_handle in file_handles {
            match file_handle.await {
                Ok(_) => {
                    info!("created Folder ") //info!("created Folder {}", &node.title);
                }
                Err(err) if err.kind() == ErrorKind::AlreadyExists => {}
                Err(_err) => {
                    error!("couldn't create folder") //error!("couldn't create folder \"{}\" - {}", &node.title, err);
                }
            }
        }
        join_all(sync_handles).await;
    })
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileWatcher {
    files: Vec<IdSize>,
    title_id_map: HashMap<String, IdSize>,
    child_to_parent_uri: HashMap<IdSize, String>,
}

impl FileWatcher {
    pub fn add_file(&mut self, id: IdSize, title: String, parent_uri: String) {
        self.title_id_map.insert(title, id);
        self.child_to_parent_uri.insert(id, parent_uri);
        self.files.push(id);
    }
    fn download_file(
        uri: &str,
        path: &PathBuf,
        client: Arc<Client<HttpsConnector<HttpConnector>>>,
    ) -> JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri.to_string())
            .header("cookie", "PHPSESSID=".to_owned() + Config::get_token())
            .body(Body::empty())
            .unwrap();
        let mut path = path.clone();
        path.set_extension("pdf");
        tokio::spawn(async move {
            let mut resp = client.request(req).await?;
            let mut file = File::create(path).await?;

            while let Some(chunk) = resp.body_mut().data().await {
                let chunk = chunk?;
                file.write_all(&chunk).await?;
            }

            Ok(())
        })
    }

    pub async fn sync(
        &mut self,
        node_tree: Arc<Mutex<IlNode>>,
        files: FileSelect,
        client: Arc<Client<HttpsConnector<HttpConnector>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let files = match files {
            FileSelect::All => &self.files,
            FileSelect::Filtered(ref vec) => vec,
        };
        let mut to_request = HashSet::new();
        for file in files {
            to_request.insert(self.child_to_parent_uri.get(file).unwrap().clone());
        }

        let mut handles: Vec<
            JoinHandle<Result<Vec<VersionInfo>, Box<dyn std::error::Error + Send + Sync>>>,
        > = vec![];
        for page in to_request {
            let client_clone = client.clone();
            handles.push(task::spawn(async move {
                let html = request_il_page(&page, client_clone.clone()).await?;
                Ok(get_versions(&html))
            }));
        }

        let mut download_handlers = vec![];
        for handle in handles {
            let version_infos = handle.await??;
            for version_info in version_infos {
                let child_node = get_node(
                    node_tree.clone(),
                    *self.title_id_map.get(&version_info.title).unwrap(),
                )
                .unwrap();
                let mut child_node = child_node.lock().unwrap();
                if let Some(sync) = &child_node.sync {
                    if &version_info.version > &sync.version {
                        download_handlers.push(FileWatcher::download_file(
                            &child_node.uri,
                            &sync.path,
                            client.clone(),
                        ));
                    }
                } else {
                    warn!("No sync Info for file {}", child_node.title);
                }
                if let Some(sync) = &mut child_node.sync {
                    info!("sync: {} info: {}", sync.version, version_info.version);
                    if &version_info.version > &sync.version {
                        sync.version = version_info.version;
                    }
                }
            }
        }
        info!("Download {} files", download_handlers.len());
        join_all(download_handlers).await;

        let mut writer = File::create("sync.ron")
            .await
            .expect("unable to create save-file");
        let s = to_string(&self).unwrap();
        let write_result = writer.write_all(s.as_bytes()).await;
        if let Err(_) = write_result {
            error!("Can't save structure.ron");
        }

        Ok(())
    }

    pub async fn new() -> Self {
        let instance = {
            match File::open("sync.ron").await {
                Ok(mut save) => {
                    let mut buffer = vec![];
                    save.read_to_end(&mut buffer).await;
                    if let Ok(file_watcher) = from_bytes(&buffer) {
                        info!("loaded sync-file");
                        Some(file_watcher)
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        }
        .unwrap_or(FileWatcher {
            files: Vec::new(),
            title_id_map: HashMap::new(),
            child_to_parent_uri: HashMap::new(),
        });

        instance
    }
}

pub enum FileSelect {
    All,
    Filtered(Vec<IdSize>),
}

struct VersionInfo {
    title: String,
    version: u16,
}

fn get_versions(elments: &Html) -> Vec<VersionInfo> {
    let mut versions = vec![];
    for container in elments.select(&CONTAINERS) {
        if let Some(node_type) = get_il_node_type(container) {
            if let Some(link) = container.select(&LINK).last() {
                let title = link.inner_html().replace("/", " ");
                if (node_type) == IlNodeType::File {
                    versions.push(VersionInfo {
                        title,
                        version: get_version(&container) as u16,
                    })
                }
            }
        }
    }
    versions
}

fn get_version(element: &ElementRef) -> u32 {
    let inner_html = element.select(&PROPERTY).nth(2).unwrap().inner_html();
    let extracted = extract_version(&inner_html).unwrap_or(1);
    extracted
}

fn extract_version(string: &str) -> Option<u32> {
    let start_index = string.find("Version: ")? + "Version: ".len();
    let end_index = start_index + string[start_index..].find("&")?;

    Some(string[start_index..end_index].parse().ok()?)
}

pub fn add_to_file_watcher(tree: &IlNode, file_watcher: &mut FileWatcher, parent_uri: String) {
    if tree.breed == IlNodeType::File {
        file_watcher.add_file(tree.id, tree.title.clone(), parent_uri)
    }
    if let Some(children) = &tree.children {
        for child in children {
            add_to_file_watcher(&child.lock().unwrap(), file_watcher, tree.uri.clone());
        }
    }
}
