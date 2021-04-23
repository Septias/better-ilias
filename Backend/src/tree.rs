use crate::client::{ClientError, IliasClient};
use futures::future::join_all;
use itertools::Itertools;
use lazy_static::lazy_static;
use log::info;
use ron::de::from_bytes;
use ron::ser::{to_string_pretty, PrettyConfig};
use scraper::{ElementRef, Selector};
use serde::{Deserialize, Serialize};
use std::{
    convert::TryInto,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{
    fs::{read_to_string, File},
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::{self, JoinHandle},
};

use crate::server::BACKEND_BASE_PATH;

const ILIAS_ROOT: &str = "ilias.php?baseClass=ilPersonalDesktopGUI&cmd=jumpToSelectedItems";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IlNode {
    pub id: u16,
    pub uri: String,
    pub title: String,
    pub breed: IlNodeType,
    pub parent: u16,
    visible: bool,
    pub children: Option<Vec<Arc<Mutex<IlNode>>>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum IlNodeType {
    Forum,
    Folder {
        sync: bool,
        store_files: bool,
        path: PathBuf,
    },
    DirectLink,
    File {
        path: PathBuf,
        version: u16,
        local: bool,
    },
    Exercise,
    Group,
    Streams,
}


impl IlNodeType {
    pub fn get_path(&mut self) -> Option<&mut PathBuf> {
        if let Self::File { path, .. } = self {
            Some(path)
        } else {
            None
        }
    }
    pub fn is_file(&self) -> bool {
        matches!(self, IlNodeType::File { .. })
    }
    pub fn get_local(&mut self) -> Option<&mut bool> {
        if let Self::File { local, .. } = self {
            Some(local)
        } else {
            None
        }
    }

}

lazy_static! {
    pub static ref CONTAINERS: Selector = Selector::parse(".ilContainerListItemOuter").unwrap();
    pub static ref LINK: Selector = Selector::parse(".il_ContainerItemTitle > a").unwrap();
    pub static ref PROPERTY: Selector = Selector::parse(".il_ItemProperty").unwrap();
    pub static ref IMAGE: Selector = Selector::parse(".ilListItemIcon").unwrap();
}

pub struct IliasTree {
    pub client: Arc<IliasClient>,
    tree: Option<Arc<Mutex<IlNode>>>,
    receiver: Mutex<Option<UnboundedReceiver<Arc<Mutex<IlNode>>>>>,
    sender: UnboundedSender<Arc<Mutex<IlNode>>>,
}

impl IliasTree {
    pub fn get_root_node(&self) -> Option<&Arc<Mutex<IlNode>>> {
        self.tree.as_ref()
    }
    pub async fn update_ilias(&self) -> Result<(), ClientError> {
        let credentials = if let Err(err) = update_ilias_tree(
            self.client.clone(),
            self.get_root_node().unwrap().clone(),
            self.sender.clone(),
        )
        .await
        .unwrap()
        {
            if let ClientError::NoToken = err {
                if let Ok(raw_credentials) = read_to_string("credentials.txt").await {
                    let credentials: [String; 2] = raw_credentials
                        .split('\n')
                        .map(|c| c.trim().to_owned())
                        .collect_vec()
                        .try_into()
                        .unwrap();
                    Ok(Some(credentials))
                } else {
                    Err(err)
                }
            } else {
                Err(err)
            }
        } else {
            Ok(None)
        }?;

        if let Some(credentials) = credentials {
            info!("Auto relogin");
            self.client.acquire_token(&credentials).await?;
            update_ilias_tree(
                self.client.clone(),
                self.get_root_node().unwrap().clone(),
                self.sender.clone(),
            )
            .await
            .unwrap()?;
        }

        Ok(())
    }

    pub async fn from_file(
        file: PathBuf,
    ) -> Result<IliasTree, Box<dyn std::error::Error + Send + Sync>> {
        if let Ok(mut file) = File::open(file).await {
            let mut buffer = vec![];
            file.read_to_end(&mut buffer).await?;
            let tree = from_bytes(&buffer)?;
            let (sender, receiver) = mpsc::unbounded_channel();
            Ok(IliasTree {
                client: Arc::new(IliasClient::new()),
                tree: Some(tree),
                receiver: Mutex::new(Some(receiver)),
                sender,
            })
        } else {
            Ok(IliasTree::new(
                &ILIAS_ROOT,
            ))
        }
    }

    pub async fn download_files(&self) -> Result<(), anyhow::Error> {
        let client = self.client.clone();
        let mut receiver = self.receiver.lock().unwrap().take().unwrap();
        while let Some(res) = receiver.recv().await {
            let client_clone = client.clone();
            tokio::spawn(async move {
                client_clone.download_file(res).await.unwrap();
            });
        }
        Ok(())
    }

    pub async fn save(&self) {
        let pretty = PrettyConfig::new()
            .with_separate_tuple_members(true)
            .with_enumerate_arrays(true);
        let mut writer = File::create("structure.ron")
            .await
            .expect("unable to create save-file");
        let s = to_string_pretty(&*self.get_root_node().unwrap().lock().unwrap(), pretty).unwrap();

        if writer.write_all(s.as_bytes()).await.is_err() {
            error!("Can't save structure.ron");
        }
    }

    pub fn new(root_node_uri: &str) -> Self {
        info!("fetching ilias_tree");
        let root_node = IlNode {
            breed: IlNodeType::Folder {
                store_files: false,
                sync: true,
                path: BACKEND_BASE_PATH.join("Studium"),
            },
            children: Some(vec![]),
            id: 0,
            parent: 0,
            title: "root_node".to_string(),
            uri: root_node_uri.to_string(),
            visible: true,
        };
        let (sender, receiver) = mpsc::unbounded_channel();
        IliasTree {
            client: Arc::new(IliasClient::new()),
            tree: Some(Arc::new(Mutex::new(root_node))),
            receiver: Mutex::new(Some(receiver)),
            sender,
        }
    }
}

struct HypNode<'a> {
    element: ElementRef<'a>,
}

impl<'a> HypNode<'a> {
    pub fn uri(&self) -> Option<&str> {
        let link = self.element.select(&LINK).last()?;
        Some(link.value().attr("href").unwrap())
    }
    fn title(&self) -> Option<String> {
        let link = self.element.select(&LINK).last()?;
        Some(link.inner_html())
    }
    fn icon_name(&self) -> Option<&str> {
        let img = self.element.select(&IMAGE).last()?;
        let img_src = img.value().attr("src")?;

        let start_index: usize = img_src.find("icon_")? + 5; 
        let end_index = start_index + img_src[start_index..].find(".svg")?;
        Some(&img_src[start_index..end_index])
    }
    fn version(&self) -> Option<usize> {
        let inner_html = self.element.select(&PROPERTY).nth(2)?.inner_html();
        let start_index = inner_html.find("Version: ")? + "Version: ".len();
        let end_index = start_index + inner_html[start_index..].find('&')?;

        inner_html[start_index..end_index].parse().ok()
    }
    pub fn compare(self, node: &mut IlNode) -> bool {
        if node.uri != self.uri().expect("can't extract uri from node") {
            false
        } else {
            match &mut node.breed {
                IlNodeType::File { version, .. } => {
                    if let Some(new_version) = self.version() {
                        if version != &(new_version as u16) {
                            *version = new_version as u16;
                            true
                        } else {
                            *version = new_version as u16;
                            false
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
    }
    pub fn into_node(self, mut path: PathBuf) -> Option<IlNode> {
        let title = self.title()?;
        
        let mut chars = title.chars();
        let start = chars.next().unwrap();
        let rest = chars.map( |character|
        match character {
            '/' | '\\' | ':'| '*'| '?'| '"' |'<'| '>' | '|' => ' ',
            _ => character.to_lowercase().next().expect(&format!("no lowercase for char {}", character))
        });

        if title == "1-1-Introduction" {
            println!("breed: {}", self.icon_name().unwrap())
        }

        let breed = match self.icon_name() {
            Some("fold") => Some(IlNodeType::Folder {
                store_files: false,
                sync: true,
                path,
            }),
            Some("crs") => Some(IlNodeType::Folder {
                store_files: false,
                sync: true,
                path,
            }),
            Some("frm") => Some(IlNodeType::Forum),
            Some("webr") => Some(IlNodeType::DirectLink),
            Some("file") => Some(IlNodeType::File {
                local: true,
                path,
                version: self.version().unwrap_or(0) as u16,
            }),
            _ => None,
        };
        Some(IlNode {
            breed: breed?,
            children: Some(vec![]),
            id: 0,
            parent: 0,
            title,
            uri: self.uri()?.to_string(),
            visible: true,
        })
    }

    pub fn new(element: ElementRef<'a>) -> Self {
        HypNode { element }
    }
}

pub fn update_ilias_tree(
    client: Arc<IliasClient>,
    node: Arc<Mutex<IlNode>>,
    file_channel: UnboundedSender<Arc<Mutex<IlNode>>>,
) -> JoinHandle<anyhow::Result<Arc<Mutex<IlNode>>, ClientError>> {
    task::spawn(async move {
        let mut handles = vec![];
        let new_children = {
            let uri = node.lock().unwrap().uri.to_string();
            let html = client.get_page(&uri).await?;

            let mut node = node.lock().unwrap();
            match node.breed.clone() {
                IlNodeType::Folder { sync, path, .. } => {
                    if let Some(children) = node.children.as_mut() {
                        if sync {
                            let elements = html.select(&CONTAINERS);

                            let new_children: Vec<Arc<Mutex<IlNode>>> = elements
                                .into_iter()
                                .map(HypNode::new)
                                .filter(|hypnode| hypnode.uri().is_some())
                                .map(|hypnode| {
                                    if let Some(node_index) = children.iter().position(|child| {
                                        child.lock().unwrap().uri == hypnode.uri().unwrap()
                                    }) {
                                        let node = children.remove(node_index);
                                        {
                                            let mut locked_node = node.lock().unwrap();
                                            if hypnode.compare(&mut locked_node) {
                                                info!("Upgraded node {}", locked_node.title);
                                                file_channel.send(node.clone()).unwrap();
                                            };
                                        }
                                        Some(node)
                                    } else if let Some(node) = hypnode.into_node(path.clone()) {
                                        Some(if node.breed.is_file() {
                                            let node = Arc::new(Mutex::new(node));
                                            file_channel.send(node.clone()).unwrap();
                                            node
                                        } else {
                                            Arc::new(Mutex::new(node))
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .flatten()
                                .collect();

                            for child in &new_children {
                                if let IlNodeType::Folder { .. } = child.lock().unwrap().breed {
                                    handles.push(update_ilias_tree(
                                        client.clone(),
                                        child.clone(),
                                        file_channel.clone(),
                                    ));
                                }
                            }

                            if !new_children.is_empty() {
                                Some(new_children)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        };

        if let Some(children) = new_children {
            node.lock().unwrap().children = Some(children);
        };

        join_all(handles).await;
        Ok(node)
    })
}

fn _set_ids(node: Arc<Mutex<IlNode>>, id: &mut u16, parent: u16) {
    let mut node = node.lock().unwrap();
    node.id = *id;
    node.parent = parent;
    *id += 1;
    if let Some(children) = &node.children {
        for child in children {
            _set_ids(child.clone(), id, node.id);
        }
    }
}
