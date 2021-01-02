use futures::future::join_all;
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;
use log::{error, info, warn};
use scraper::{ElementRef, Html};
use std::{collections::{HashMap, HashSet}, io::ErrorKind, path::PathBuf, sync::{Arc, Mutex}};
use tokio::{fs::create_dir, task::{self, JoinHandle}};

use crate::{IdSize, helpers::{get_node, request_il_page}, tree::{get_il_node_type, IlNode, IlNodeType, CONTAINERS, LINK, PROPERTY}};

pub fn sync(
    node: &'static IlNode,
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut sync_handles = vec![];
        match node.breed {
            IlNodeType::Folder => {
                if let Some(sync_info) = node.sync {
                    match create_dir(&sync_info.path).await {
                        Ok(_) => {
                            info!("created Folder {}", &node.title)
                        }
                        Err(err) if err.kind() == ErrorKind::AlreadyExists => {}
                        Err(err) => {
                            error!("couldn't create folder \"{}\" - {} - {:?}", &node.title, err, sync_info.path);
                        }
                    }
                    if let Some(children) = &node.children {
                        for child in children
                            .iter()
                            .filter(|child| child.breed == IlNodeType::Folder)
                        {
                            sync_handles.push(sync(child, client.clone()));
                        }
                    }
                }
            }
            IlNodeType::File => {}
            _ => (),
        }
        join_all(sync_handles).await;
    })
}

#[derive(Debug)]
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
    fn download_file(uri: &str, path: &PathBuf) -> JoinHandle<()> {
        let uri = uri.to_string();
        tokio::spawn(async move { 
            info!("download file {}", uri);
        })
    }

    pub async fn sync(
        &mut self,
        node_tree: &mut IlNode,
        files: FileSelect,
        client: Arc<Client<HttpsConnector<HttpConnector>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        let files = match files {
            FileSelect::All => {
                &self.files
            }
            FileSelect::Filtered(ref vec) => {
                vec
            }
        };
        let mut to_request = HashSet::new();
        for file in files {
            to_request.insert(self.child_to_parent_uri.get(file).unwrap().clone());
        }

        let mut handles: Vec<JoinHandle<Result<Vec<VersionInfo>, Box<dyn std::error::Error + Send + Sync>>>> = vec![];
        for page in to_request {
            let client_clone = client.clone();
            handles.push(task::spawn(async move {
                let html = request_il_page(&page, client_clone.clone()).await?;
                Ok(get_versions(&html))
            }));
        }

        let mut download_handlers = vec![];
        for handle in handles{
            let version_infos = handle.await??;
            for version_info in version_infos {
                let child_node = get_node(node_tree, *self.title_id_map.get(&version_info.title).unwrap()).unwrap();
                if &version_info.version > &child_node.id{
                    if let Some(sync) = &child_node.sync {
                        download_handlers.push(FileWatcher::download_file(&child_node.uri, &sync.path));
                    } else {
                        warn!("No sync Info for file {}", child_node.title);
                    }
                    
                }
            }
        }

        Ok(()) // ot
    }

    pub fn new() -> Self {
        return {
            FileWatcher {
                files:Vec::new(),
                title_id_map: HashMap::new(),
                child_to_parent_uri: HashMap::new(),
            }
        };
    }
}

pub enum FileSelect{
    All,
    Filtered(Vec<IdSize>)
    
}

struct VersionInfo {
    title: String,
    version: u16,
}

fn get_versions(elments: &Html) -> Vec<VersionInfo> {
    let mut versions = vec![];
    for container in elments.select(&CONTAINERS) {
        let element = container.select(&LINK).last().unwrap();
        let uri = element.value().attr("href").unwrap();
        let title = element.inner_html().replace("/", " ");
        if get_il_node_type(uri).unwrap() == IlNodeType::File {
            versions.push(VersionInfo {
                title,
                version: get_version(&container) as u16
            })
        }
    }
    versions
}


fn get_version(element: &ElementRef) -> u32 {
    let inner_html = element.select(&PROPERTY).nth(2).unwrap().inner_html();
    let extracted = extract_version(&inner_html).unwrap_or(1);
    println!("{}: {}", inner_html.trim(), extracted);
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
            add_to_file_watcher(child, file_watcher, tree.uri.clone());
        } 
    }
}