use hyper::{Client, client::{HttpConnector}};
use hyper_tls::HttpsConnector;
use log::{error, info};
use ron::{
    de::from_bytes,
    ser::{to_string_pretty, PrettyConfig},
};
use scraper::{Selector};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    task::{self, JoinHandle},
};
use lazy_static::lazy_static;

use crate::{FileWatcher, IdSize, helpers::request_il_page, sync::add_to_file_watcher};


#[derive(Debug, Deserialize, Serialize)]
pub struct SyncInfo{
    pub path: PathBuf,
    pub version: u32
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IlNode {
    pub title: String,
    pub id: u16,
    pub uri: String,
    pub sync: Option<SyncInfo>, // should this node be synced
    pub breed: IlNodeType,
    pub children: Option<Vec<IlNode>>,
    pub parent: u16
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum IlNodeType {
    Forum,
    Folder,
    DirectLink,
    File,
}

pub fn get_il_node_type(uri: &str) -> Option<IlNodeType> {
    let cmd = uri
        .split("&")
        .find_map(|urlpiece| urlpiece.strip_prefix("cmd="));
    match cmd {
        Some("view") => Some(IlNodeType::Folder),
        Some("showThreads") => Some(IlNodeType::Forum),
        Some("calldirectlink") => Some(IlNodeType::DirectLink),
        Some(_) => None,
        None => {
            if uri.contains("goto.php") {
                Some(IlNodeType::File)
            } else {
                None
            }
        }
    }
}

struct FolderInfo{
    uri: String,
    title: String
}

lazy_static!{
    pub static ref CONTAINERS: Selector = Selector::parse(".ilContainerListItemOuter").unwrap();
    pub static ref LINK: Selector = Selector::parse(".il_ContainerItemTitle > a").unwrap();
    pub static ref PROPERTY: Selector = Selector::parse(".il_ItemProperty").unwrap();
}

pub fn create_ilias_tree(
    uri: String,
    title: String, 
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
    path: PathBuf,
) -> JoinHandle<IlNode> {
    task::spawn(async move {
        let mut cloned = path.clone();
        cloned.push(&title);
        let mut node = IlNode {
            title,
            children: None,
            sync: Some(SyncInfo{
                path: cloned,
                version: 0
            }),
            breed: IlNodeType::Folder,
            uri: uri.clone(),
            id: 0,
            parent: 0
        };
        
        let mut children = vec![];
        let folders = {
            
            let html = request_il_page(&uri, client.clone()).await.unwrap();
            let elements = html.select(&CONTAINERS);

            // create children
            let mut folders = vec![];
            // go through all possible folders
            for element in elements {
                // if it has a link field it actually is a folder
                if let Some(element) = element.select(&LINK).last() {
                    
                    let child_uri = element.value().attr("href").unwrap();
                    let title = element.inner_html().replace("/", " ");
                    if let Some(node_type) = get_il_node_type(child_uri){

                        if &node_type == &IlNodeType::Folder{
                            folders.push(FolderInfo {
                                uri: child_uri.to_string(),
                                title
                            })
                        } else {
                            let mut temp_path = path.clone();
                            temp_path.push(&title);
                            let mut node = IlNode{
                                breed: node_type.clone(),
                                uri: child_uri.to_string(),
                                children: None,
                                id: 0,
                                sync: None,
                                title: title.clone(),
                                parent: 0
                            };
                            if &node_type == &IlNodeType::File {
                                node.sync = Some(SyncInfo{
                                    path: temp_path,
                                    version: 0
                                });
                            }

                            children.push(node)
                            
                        }
                    }
                } 
            }
            folders
        };
        let mut handles = vec![];

        for folder in folders {
            handles.push(create_ilias_tree(
                folder.uri,
                folder.title,
                client.clone(),
                path.clone()
            ));
        }
        
        // load sub-folders and add them to children
        for handle in handles {
            if let Ok(child) = handle.await {
                children.push(child);
            }
        }
        node.children = Some(children);
        node
    })
}

fn set_ids(node: &mut IlNode, id: &mut IdSize, parent: IdSize) {
    node.id = id.clone();
    node.parent = parent;
    *id += 1;
    if let Some(children) = node.children.as_mut() {
        for child in children.iter_mut() {
            set_ids(child, id, node.id);
        }
    }
}

pub async fn get_or_create_ilias_tree(
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
    file_watcher: &mut FileWatcher
) -> Result<IlNode, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(ilias_tree) = match File::open("structure.ron").await {
        Ok(mut save) => {
            let mut buffer = vec![];
            save.read_to_end(&mut buffer).await?;
            if let Ok(ilias_tree) = from_bytes(&buffer) {
                Some(ilias_tree)
            } else {
                None
            }
        }
        Err(_) => None,
    } {
        info!("loaded ilias_tree from file");
        Ok(ilias_tree)
    } else {
        info!("fetching ilias_tree");
        let mut ilias_tree = create_ilias_tree(
                "ilias.php?ref_id=1836117&cmdClass=ilrepositorygui&cmdNode=yj&baseClass=ilrepositorygui"
                .to_string(),
            "Rechnernetze".to_string(),
            client,
            PathBuf::from("Rechnernetze")
        ).await?;

        set_ids(&mut ilias_tree, &mut 0, 0);
        add_to_file_watcher(&ilias_tree, file_watcher, "Bischte Dumm".to_string());
        
        // save to file
        let pretty = PrettyConfig::new()
            .with_separate_tuple_members(true)
            .with_enumerate_arrays(true);
        let mut writer = File::create("structure.ron")
            .await
            .expect("unable to create save-file");
        let s = to_string_pretty(&ilias_tree, pretty).unwrap();
        let write_result = writer.write_all(s.as_bytes()).await;
        if let Err(_) = write_result {
            error!("Can't save structure.ron");
        }
        Ok(ilias_tree)
    }
}

