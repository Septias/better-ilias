use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;
use log::{error, info};
use ron::{
    de::from_bytes,
    ser::{to_string_pretty, PrettyConfig},
};
use scraper::{ElementRef, Selector};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    task::{self, JoinHandle},
};

use crate::{helpers::request_il_page, sync::add_to_file_watcher, FileWatcher, IdSize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncInfo {
    pub path: PathBuf,
    pub version: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IlNode {
    pub title: String,
    pub id: u16,
    pub uri: String,
    pub sync: Option<SyncInfo>, // should this node be synced
    pub breed: IlNodeType,
    pub children: Option<Vec<Arc<Mutex<IlNode>>>>,
    pub parent: u16,
    visible: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum IlNodeType {
    Forum,
    Folder,
    DirectLink,
    File,
}

pub fn get_il_node_type(element: ElementRef) -> Option<IlNodeType> {
    let img = element.select(&IMAGE).last()?;
    let img_src = img.value().attr("src")?;

    const START_INDEX: usize = 32; // "./templates/default/images/icon_fold.svg" icon_ ends at 31
    let end_index = START_INDEX + img_src[START_INDEX..].find(".svg")?;

    match &img_src[START_INDEX..end_index] {
        "fold" => Some(IlNodeType::Folder),
        "crs" => Some(IlNodeType::Folder),
        "frm" => Some(IlNodeType::Forum),
        "webr" => Some(IlNodeType::DirectLink),
        "file" => Some(IlNodeType::File),
        _ => None,
    }
}

#[cfg(test)]
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

struct FolderInfo {
    uri: String,
    title: String,
}

lazy_static! {
    pub static ref CONTAINERS: Selector = Selector::parse(".ilContainerListItemOuter").unwrap();
    pub static ref LINK: Selector = Selector::parse(".il_ContainerItemTitle > a").unwrap();
    pub static ref PROPERTY: Selector = Selector::parse(".il_ItemProperty").unwrap();
    pub static ref IMAGE: Selector = Selector::parse(".ilListItemIcon").unwrap();
}

pub fn create_ilias_tree(
    uri: String,
    title: String,
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
    path: PathBuf,
) -> JoinHandle<Arc<Mutex<IlNode>>> {
    task::spawn(async move {
        let mut path = path.clone();
        path.push(&title);
        let node = Arc::new(Mutex::new(IlNode {
            visible: true,
            title,
            children: None,
            sync: Some(SyncInfo {
                path: path.clone(),
                version: 0,
            }),
            breed: IlNodeType::Folder,
            uri: uri.clone(),
            id: 0,
            parent: 0,
        }));

        let mut children = vec![];
        let folders = {
            let html = request_il_page(&uri, client.clone()).await.unwrap();
            let elements = html.select(&CONTAINERS);

            // create children
            let mut folders = vec![];
            // go through all possible folders
            for element in elements {
                // if it has a link field it actually is a folder
                if let Some(node_type) = get_il_node_type(element) {
                    if let Some(link) = element.select(&LINK).last() {
                        let child_uri = link.value().attr("href").unwrap();
                        let title = link.inner_html().replace("/", " ");
                        if &node_type == &IlNodeType::Folder {
                            folders.push(FolderInfo {
                                uri: child_uri.to_string(),
                                title,
                            })
                        } else {
                            let mut path = path.clone();
                            path.push(&title);
                            let mut node = IlNode {
                                breed: node_type.clone(),
                                uri: child_uri.to_string(),
                                children: None,
                                id: 0,
                                sync: None,
                                title: title.clone(),
                                parent: 0,
                                visible: true,
                            };
                            if &node_type == &IlNodeType::File {
                                node.sync = Some(SyncInfo {
                                    path: path,
                                    version: 0,
                                });
                            }

                            children.push(Arc::new(Mutex::new(node)));
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
                path.clone(),
            ));
        }

        // load sub-folders and add them to children
        for handle in handles {
            if let Ok(child) = handle.await {
                children.push(child);
            }
        }

        node.lock().unwrap().children = Some(children);
        node
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

pub async fn get_or_create_ilias_tree(
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
    file_watcher: &mut FileWatcher,
) -> Result<Arc<Mutex<IlNode>>, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(ilias_tree) = match File::open("structure.ron").await {
        Ok(mut save) => {
            let mut buffer = vec![];
            save.read_to_end(&mut buffer).await?;
            if let Ok(ilias_tree) = from_bytes(&buffer) {
                Some(Arc::new(Mutex::new(ilias_tree)))
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
        let ilias_tree = create_ilias_tree(
            "ilias.php?baseClass=ilPersonalDesktopGUI&cmd=jumpToMemberships".to_string(),
            "Studium".to_string(),
            client,
            PathBuf::from(""),
        )
        .await?;

        set_ids(ilias_tree.clone(), &mut 0, 0);
        add_to_file_watcher(
            &ilias_tree.clone().lock().unwrap(),
            file_watcher,
            "Bischte Dumm".to_string(),
        );

        let pretty = PrettyConfig::new()
            .with_separate_tuple_members(true)
            .with_enumerate_arrays(true);
        let mut writer = File::create("structure.ron")
            .await
            .expect("unable to create save-file");
        let s = to_string_pretty(&*ilias_tree.lock().unwrap(), pretty).unwrap();
        let write_result = writer.write_all(s.as_bytes()).await;
        if let Err(_) = write_result {
            error!("Can't save structure.ron");
        }
        Ok(ilias_tree)
    }
}
