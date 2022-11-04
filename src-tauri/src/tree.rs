use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use futures::future::join_all;
use lazy_static::lazy_static;

use scraper::{ElementRef, Selector};
use tokio::{
    task::{JoinHandle},
};

use crate::{
    client::{IliasClient},
    ilias::{IlNode, IlNodeType},
};

lazy_static! {
    pub static ref CONTAINERS: Selector = Selector::parse(".ilContainerListItemOuter").unwrap();
    pub static ref LINK: Selector = Selector::parse(".il_ContainerItemTitle > a").unwrap();
    pub static ref PROPERTY: Selector = Selector::parse(".il_ItemProperty").unwrap();
    pub static ref IMAGE: Selector = Selector::parse(".ilListItemIcon").unwrap();
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
        let rest = chars.map(|character| match character {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => ' ',
            _ => character
                .to_lowercase()
                .next()
                .unwrap_or_else(|| panic!("no lowercase for char {}", character)),
        });
        path.push(std::iter::once(start).chain(rest).collect::<String>());

        let breed = match self.icon_name() {
            Some("fold") => Some(IlNodeType::Folder {
                store_files: false,
                path,
            }),
            Some("crs") => Some(IlNodeType::Folder {
                store_files: false,
                path,
            }),
            Some("frm") => Some(IlNodeType::Forum),
            Some("webr") => Some(IlNodeType::DirectLink),
            Some("file") => Some(IlNodeType::File {
                local: false,
                path,
                version: self.version().unwrap_or(0) as u16,
            }),
            Some("xvid") => Some(IlNodeType::Video),
            Some("exc") => Some(IlNodeType::Exercise),
            _ => None,
        };
        Some(IlNode {
            breed: breed?,
            children: Some(vec![]),
            title,
            uri: self.uri()?.to_string(),
            visible: true,
        })
    }

    pub fn new(element: ElementRef<'a>) -> Self {
        HypNode { element }
    }
}

pub fn update_node(
    client: Arc<IliasClient>,
    node: Arc<Mutex<IlNode>>,
) -> JoinHandle<anyhow::Result<Arc<Mutex<IlNode>>>> {
    tokio::spawn(async move {
        let mut child_handles = vec![];
        let mut download_handles = vec![];

        let (uri, children, path) = {
            let mut node = node.lock().unwrap();
            (node.uri.clone(), node.children.take(), node.breed.path())
        };

        let new_children: Vec<Arc<Mutex<IlNode>>> = if let Some(mut children) = children {
            let html = client.get_page(&uri).await?;
            let elements = html.select(&CONTAINERS);

            // build new children from fresh children list
            elements
                .into_iter()
                .map(HypNode::new)
                .filter(|hypnode| hypnode.uri().is_some())
                .filter_map(|hypnode| {
                    // try to find child in old children
                    let position = children
                        .iter()
                        .position(|child| child.lock().unwrap().uri == hypnode.uri().unwrap());

                    // if we find the child we replace it
                    if let Some(node_index) = position {
                        let node = children.remove(node_index);
                        let change = hypnode.compare(&mut node.lock().unwrap());
                        if change {
                            let node = node.clone();
                            let client = client.clone();
                            download_handles.push(tokio::spawn(async move {
                                client.download_file(node).await
                            }));
                        }
                        Some(node)
                    } else { hypnode.into_node(
                        path.as_ref()
                            .expect("node with children must have path")
                            .clone(),
                    ).map(|node| Arc::new(Mutex::new(node))) }
                })
                .collect()
        } else {
            vec![]
        };

        for child in &new_children {
            match child.lock().unwrap().breed.clone() {
                IlNodeType::Folder { .. } => {
                    child_handles.push(update_node(client.clone(), child.clone()));
                }
                IlNodeType::DirectLink => {
                    let child_clone = child.clone();
                    let client_clone = client.clone();
                    tokio::spawn(async move {
                        client_clone.flatten_link(&child_clone).await.unwrap();
                    });
                }
                _ => {}
            }
        }
        join_all(child_handles).await;
        join_all(download_handles).await;
        Ok(node)
    })
}
