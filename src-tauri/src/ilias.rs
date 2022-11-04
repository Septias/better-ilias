use crate::{client::IliasClient, tree::update_node};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, read_to_string},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::UnboundedSender;

const ILIAS_ROOT: &str =
    "ilias.php?cmdClass=ilmembershipoverviewgui&cmdNode=kt&baseClass=ilmembershipoverviewgui";

type WrappedNode = Arc<Mutex<IlNode>>;
pub type Somewhat = String;
type Credentials = [String; 2];
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IlNode {
    pub uri: String,
    pub title: String,
    pub breed: IlNodeType,
    pub visible: bool,
    pub children: Option<Vec<WrappedNode>>,
}

impl Default for IlNode {
    fn default() -> Self {
        Self {
            uri: ILIAS_ROOT.to_string(),
            title: "Root".to_string(),
            breed: IlNodeType::Root,
            visible: true,
            children: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum IlNodeType {
    Forum,
    Folder {
        store_files: bool,
        path: PathBuf,
    },
    DirectLink,
    File {
        path: PathBuf,
        version: u16,
        local: bool,
    },
    Video,
    Exercise,
    Group,
    Root,
}

impl IlNodeType {
    pub fn path_mut(&mut self) -> Option<&mut PathBuf> {
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

    pub fn path(&self) -> Option<PathBuf> {
        if let IlNodeType::Folder { path, .. } = self {
            Some(path.clone())
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct IliasTree {
    #[serde(skip)]
    client: Option<Arc<IliasClient>>,
    tree: Mutex<Option<IlNode>>,
    credentials: Option<Credentials>,
}

impl IliasTree {
    pub async fn new() -> Self {
        if let Ok(save_data) = read_to_string("path") {
            serde_json::from_str(&save_data).unwrap()
        } else {
            IliasTree {
                client: None,
                tree: Mutex::new(Some(IlNode::default())),
                credentials: None,
            }
        }
    }

    pub async fn run(self) {
        // let (sender, receiver) = mpsc::unbounded_channel();

        /* while let Some(res) = receiver.recv().await {
            let client_clone = client.clone();
            tokio::spawn(async move {
                client_clone.download_file(res).await.unwrap();
            });
        } */
    }

    pub async fn update_root(&self) -> anyhow::Result<()> {
        if let Some(client) = &self.client {
            let node = Arc::new(Mutex::new(self.tree.lock().unwrap().take().unwrap()));
            update_node(client.clone(), node).await?;
        }
        Ok(())
    }

    pub async fn save(&self) {
        fs::write("save.json", serde_json::to_string(&self).unwrap()).unwrap()
    }
}
