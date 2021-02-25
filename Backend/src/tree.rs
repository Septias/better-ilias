use crate::{client::IliasClient, IdSize};
use futures::{future::join_all};
use lazy_static::lazy_static;
use log::{info, warn};
use ron::de::from_bytes;
use scraper::{ElementRef, Selector};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf, sync::{Arc, Mutex}};
use tokio::{fs::File, io::AsyncReadExt, sync::mpsc::{UnboundedReceiver , UnboundedSender}, task::{self, JoinHandle}};

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
    pub fn get_path(&mut self) -> Option<&mut PathBuf>{
        if let Self::File{path, ..} = self {
            Some(path)
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


#[derive(Debug)]
enum IliasError {
    NoTree,
}

impl Display for IliasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IliasError::NoTree => {
                write!(f, "No IliasTree available")
            }
        }
    }
}

impl std::error::Error for IliasError {}

pub struct ILiasTree {
    client: Arc<IliasClient>,
    tree: Option<IlNode>,
}


impl ILiasTree {
    pub fn get_root_node(&self) -> Option<&IlNode> {
        self.tree.as_ref()
    }
    pub async fn update_ilias(
        &self,
        file_channel: UnboundedSender<Arc<Mutex<IlNode>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        /*for child in self
            .get_root_node()
            .ok_or(IliasError::NoTree)?
            .children
            .as_ref()
            .expect("root node has no children")
        {
            update_ilias_tree(self.client.clone(), child.clone(), file_channel.clone()).await??;
        } */


        update_ilias_tree(self.client.clone(), Arc::new(Mutex::new(self.tree.clone().expect("no root node"))), file_channel.clone()).await??;

        Ok(())
    }
    pub fn _get_node(&self) -> Option<&IlNode> {
        self.tree.as_ref()
    }
    pub async fn from_file(
        file: PathBuf,
    ) -> Result<ILiasTree, Box<dyn std::error::Error + Send + Sync>> {
        if let Ok(mut file) = File::open(file).await{
            let mut buffer = vec![];
            file.read_to_end(&mut buffer).await?;
            let tree = from_bytes(&buffer)?;

            Ok(ILiasTree {
                client: Arc::new(IliasClient::new()),
                tree: Some(tree)
            })
        } else {
            Ok(ILiasTree::new(&"Studium"))
        }  
    }

    pub fn download_files(
        &self,
        mut receiver: UnboundedReceiver<Arc<Mutex<IlNode>>>,
    ) -> JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {
        let client = self.client.clone();
        tokio::spawn( async move {
            while let Some(res) = receiver.recv().await { //remove this unwrap
                client.download_file(res).await?;
            };
            Ok(())
        })
    
    }

    pub fn new(root_node_uri: &str) -> Self {
        info!("fetching ilias_tree");
        let root_node = IlNode {
            breed: IlNodeType::Folder {
                store_files: false,
                sync: true,
                path: PathBuf::new(),
            },
            children: Some(vec![]),
            id: 0,
            parent: 0,
            title: "root_node".to_string(),
            uri: root_node_uri.to_string(),
            visible: true,
        };
        ILiasTree {
            client: Arc::new(IliasClient::new()),
            tree: Some(root_node),
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

        const START_INDEX: usize = 32; // "./templates/default/images/icon_fold.svg" icon_ ends at 31
        let end_index = START_INDEX + img_src[START_INDEX..].find(".svg")?;
        Some(&img_src[START_INDEX..end_index])
    }
    fn version(&self) -> Option<usize> {
        let inner_html = self.element.select(&PROPERTY).nth(2).unwrap().inner_html();
        let start_index = inner_html.find("Version: ")? + "Version: ".len();
        let end_index = start_index + inner_html[start_index..].find("&")?;

        Some(inner_html[start_index..end_index].parse().ok()?)
    }
    pub fn compare(self, node: &mut IlNode) -> bool {
        if node.uri != self.title().expect("can't extract uri from node") {
            false
        } else {
            match node.breed {
                IlNodeType::File { mut version, .. } => {
                    if let Some(new_version) = self.version() {
                        version = new_version as u16;
                    } else {
                        warn!("unable to extract version of {}", node.title);
                    }
                }
                _ => (),
            };
            true
        }
    }
    pub fn to_node(self) -> Option<IlNode> {
        let path = PathBuf::new();
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
                local: false,
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
            title: self.title()?,
            uri: self.uri()?.to_string(),
            visible: true,
        })
    }

    pub fn new(element: ElementRef<'a>) -> Self {
        HypNode {
            element
        }
    }
}

pub fn update_ilias_tree(
    client: Arc<IliasClient>,
    node: Arc<Mutex<IlNode>>,
    file_channel: UnboundedSender<Arc<Mutex<IlNode>>>,
) -> JoinHandle<Result<Arc<Mutex<IlNode>>, Box<dyn std::error::Error + Send + Sync>>> {
    task::spawn(async move {
        let mut handles = vec![];
        let new_children= {
            let uri = node.lock().unwrap().uri.to_string(); 
            let html = client.get_page(&uri).await?;
            match node.lock().unwrap().breed {
                IlNodeType::Folder { sync, .. } => {
                    if let Some(children) = node.lock().unwrap().children.as_mut() {
                        if sync {
                            let elements = html.select(&CONTAINERS);

                            
                            let new_children: Vec<Arc<Mutex<IlNode>>> = elements.into_iter()
                                .map(|element| HypNode::new(element))
                                .filter(|hypnode| hypnode.uri().is_some())
                                .map(| hypnode | {
                                if let Some(node_index) = children.iter().position(
                                    |child| &child.lock().unwrap().uri == hypnode.uri().unwrap()) {
                                        let node = children.remove(node_index);
                                        {
                                            let mut locked_node = node.lock().unwrap();
                                            hypnode.compare( &mut locked_node);
                                        }
                                        node
                                } else {
                                    Arc::new(Mutex::new(hypnode.to_node().unwrap()))
                                }
                            }).collect();

                            for child in &new_children {
                                if let IlNodeType::Folder { .. } = child.lock().unwrap().breed {
                                    handles.push(update_ilias_tree(
                                        client.clone(),
                                        child.clone(),
                                        file_channel.clone(),
                                    ));
                                }
                            }

                            if new_children.len() > 0{
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
                _ => None
            }
        };

        if let Some(children) = new_children {
            node.lock().unwrap().children = Some(children);
        };

        join_all(handles).await;
        Ok(node)
    })
}

fn set_ids(node: Arc<Mutex<IlNode>>, id: &mut IdSize, parent: IdSize) {
    let mut node = node.lock().unwrap();
    node.id = id.clone();
    node.parent = parent;
    *id += 1;
    if let Some(children) = &node.children {
        for child in children {
            set_ids(child.clone(), id, node.id);
        }
    }
}

/* #[cfg(test)]
mod tests {
    use super::{get_il_node_type, IlNodeType, Selector};
    use scraper::{ElementRef, Html};

    fn get_element(folder: &Html) -> ElementRef<'_> {
        let root_div_sel = Selector::parse(".ilCLI.ilObjListRow.row").unwrap();
        folder.select(&root_div_sel).last().unwrap()
    }
    fn get_html(file: &str) -> Html {
        let html: String =
            String::from_utf8_lossy(&std::fs::read("test_html/".to_string() + file).unwrap())
                .parse()
                .unwrap();
        Html::parse_fragment(&html)
    }
    #[test]
    fn test_identify_folder() {
        let html = get_html("folder.html");
        let element = get_element(&html);
        assert_eq!(get_il_node_type(element), Some(IlNodeType::Folder));
    }
    #[test]
    fn test_identify_file() {
        let html = get_html("file.html");
        let element = get_element(&html);
        assert_eq!(get_il_node_type(element), Some(IlNodeType::File));
    }
    #[test]
    fn test_identify_forum() {
        let html = get_html("forum.html");
        let element = get_element(&html);
        assert_eq!(get_il_node_type(element), Some(IlNodeType::Forum));
    }
    #[test]
    fn test_identify_direct_link() {
        let html = get_html("direct_link.html");
        let element = get_element(&html);
        assert_eq!(get_il_node_type(element), Some(IlNodeType::DirectLink));
    }
}
 */
