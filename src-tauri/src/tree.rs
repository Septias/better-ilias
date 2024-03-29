use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use futures::future::join_all;
use lazy_static::lazy_static;

use scraper::{ElementRef, Selector};
use serde::Serialize;
use thiserror::Error;
use tokio::task::JoinHandle;

use crate::{
    client::{ClientError, IliasClient},
    ilias::{IlNode, IlNodeType, ILIAS_ROOT, ROOT_PATH},
};

lazy_static! {
    // selectors for content of one course
    pub static ref CONTAINERS: Selector = Selector::parse(".ilContainerListItemOuter").unwrap();
    pub static ref LINK: Selector = Selector::parse(".il_ContainerItemTitle > a").unwrap();
    pub static ref PROPERTY: Selector = Selector::parse(".il_ItemProperty").unwrap();
    pub static ref IMAGE: Selector = Selector::parse(".ilListItemIcon").unwrap();

    // selectors for root-node
    pub static ref ROOT_CONTAINERS: Selector = Selector::parse(".il-item").unwrap();
    pub static ref ROOT_IMAGE: Selector = Selector::parse(".icon").unwrap();
    pub static ref ROOT_LINK: Selector = Selector::parse(".il-item-title > a").unwrap();

}

#[derive(Debug)]
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
    pub fn same_version(self, node: &mut IlNode) -> bool {
        if node.uri == self.uri().expect("can't extract uri from node") {
            true
        } else {
            match &mut node.breed {
                IlNodeType::File { version, .. } => {
                    if let Some(new_version) = self.version() {
                        if *version == new_version {
                            true
                        } else {
                            *version = new_version;
                            false
                        }
                    } else {
                        true
                    }
                }
                _ => true,
            }
        }
    }
    pub fn into_node(self, mut path: PathBuf) -> Option<IlNode> {
        let title = self.title()?;

        let mut chars = title.chars();
        let start = chars.next().unwrap();
        let rest = chars
            .filter_map(|character| match character {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => None,
                ' ' => Some('_'),
                c => Some(c),
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
                local: true,
                path,
                version: self.version().unwrap_or(0),
            }),
            Some("file_inline") => Some(IlNodeType::File {
                local: true,
                path,
                version: self.version().unwrap_or(0),
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
) -> JoinHandle<Result<Arc<Mutex<IlNode>>, TreeError>> {
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

                    // if we find the child we might replace it
                    if let Some(node_index) = position {
                        let node = children.remove(node_index);
                        let same_node = hypnode.same_version(&mut node.lock().unwrap());
                        if !same_node {
                            let node = node.clone();
                            let client = client.clone();
                            download_handles.push(tokio::spawn(async move {
                                client.download_file(node).await
                            }));
                        }
                        Some(node)
                    } else {
                        if let Some(node) = hypnode.into_node(
                            path.as_ref()
                                .expect("program logic shoul ensure this")
                                .clone(),
                        ) {
                            let node = Arc::new(Mutex::new(node));
                            let node_clone = node.clone();
                            if node.lock().unwrap().breed.is_file() {
                                let client = client.clone();
                                download_handles.push(tokio::spawn(async move {
                                    client.download_file(node_clone).await
                                }));
                            };
                            Some(node)
                        } else {
                            None
                        }
                    }
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
        if new_children.len() > 0 {
            node.lock().unwrap().children = Some(new_children);
        } else {
            node.lock().unwrap().children = None;
        }
        Ok(node)
    })
}

pub fn update_root(
    client: Arc<IliasClient>,
    root: Arc<Mutex<IlNode>>,
) -> JoinHandle<Result<(), TreeError>> {
    let mut root_children = root.lock().unwrap().children.take();
    tokio::spawn(async move {
        let children = {
            let html = client.get_page(ILIAS_ROOT).await?;
            let elements = html.select(&ROOT_CONTAINERS);
            elements
                .filter(is_kurs)
                .map(|elem| {
                    let link = elem.select(&ROOT_LINK).next().unwrap();
                    let uri = link.value().attr("href").unwrap().to_string();
                    if let Some(children) = &mut root_children {
                        if let Some(position) = children
                            .iter()
                            .position(|node| node.lock().unwrap().uri == uri)
                        {
                            return children.remove(position);
                        }
                    }

                    let title = link.inner_html();
                    let folder = title
                        .chars()
                        .filter_map(|character| match character {
                            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => None,
                            ' ' => Some('_'),
                            c => Some(c),
                        })
                        .collect::<String>();
                    Arc::new(Mutex::new(IlNode {
                        uri,
                        breed: IlNodeType::Folder {
                            store_files: true,
                            // This is done sooo fishy xD
                            path: Some(PathBuf::from(ROOT_PATH))
                                .map(|mut path| {
                                    path.push(folder);
                                    path
                                })
                                .unwrap(),
                        },
                        title,
                        visible: true,
                        children: Some(vec![]),
                    }))
                })
                .collect::<Vec<_>>()
        };

        let handles = children
            .iter()
            .map(|child| update_node(client.clone(), child.clone()));
        join_all(handles).await;

        if children.len() > 0 {
            root.lock().unwrap().children = Some(children);
        } else {
            root.lock().unwrap().children = None;
        };
        Ok(())
    })
}

fn is_kurs(element: &ElementRef) -> bool {
    element
        .select(&ROOT_IMAGE)
        .next()
        .unwrap()
        .value()
        .attr("alt")
        .unwrap()
        .contains("Kurs")
}

#[derive(Debug, Error, Serialize)]
pub enum TreeError {
    #[error(transparent)]
    Client(#[from] ClientError),
}
