use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;
use log::{error, info};
use ron::{
    de::from_bytes,
    ser::{to_string_pretty, PrettyConfig},
};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    task::{self, JoinHandle},
};

use crate::sync::FileWatcher;

#[derive(Debug, Deserialize, Serialize)]
pub struct IlNode {
    pub title: String,
    pub id: u16,
    pub uri: String,
    pub sync: bool, // should this node be synced
    pub breed: IlNodeType,
    pub children: Option<Vec<IlNode>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum IlNodeType {
    Forum,
    Folder,
    DirectLink,
    File,
}

fn get_il_node_type(uri: &str) -> Option<IlNodeType> {
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

pub fn create_ilias_tree(
    uri: String,
    title: String,
    _client: Arc<Client<HttpsConnector<HttpConnector>>>,
    _file_watcher: Arc<FileWatcher>,
) -> JoinHandle<IlNode> {
    task::spawn(async move {
        let node = IlNode {
            title,
            children: None,
            sync: false,
            breed: IlNodeType::Folder,
            uri: uri.clone(),
            id: 0,
        };
        node
    })
}

fn set_ids(node: &mut IlNode, id: &mut u16) {
    node.id = id.clone();
    *id += 1;
    if let Some(children) = node.children.as_mut() {
        for child in children.iter_mut() {
            set_ids(child, id);
        }
    }
}

pub async fn get_or_create_ilias_tree(
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
    file_watcher: Arc<FileWatcher>,
) -> Result<IlNode, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(ilias_tree) = match File::open("structure.ron").await {
        Ok(mut save) => {
            let mut buffer = vec![];
            save.read_to_end(&mut buffer).await;
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
                "ilias.php?ref_id=1843349&cmd=view&cmdClass=ilrepositorygui&cmdNode=yj&baseClass=ilrepositorygui"//"ilias.php?ref_id=1836117&cmdClass=ilrepositorygui&cmdNode=yj&baseClass=ilrepositorygui&cmd=view"
                .to_string(),
            "Rechnernetze".to_string(),
            client,
            file_watcher
        ).await?;

        set_ids(&mut ilias_tree, &mut 0);

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
