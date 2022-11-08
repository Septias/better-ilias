use crate::{
    client::{ClientError, Credentials, IliasClient},
    tree::{update_node, TreeError},
};
use futures::{SinkExt, StreamExt};
use log::info;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, read_to_string},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use tokio::{
    net::{TcpListener, TcpStream},
    signal,
};

const ILIAS_ROOT: &str =
    "ilias.php?cmdClass=ilmembershipoverviewgui&cmdNode=kt&baseClass=ilmembershipoverviewgui";

type WrappedNode = Arc<Mutex<IlNode>>;
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

#[derive(Clone)]
pub struct IliasTree {
    tree: WrappedNode,
    client: Arc<Mutex<Option<Arc<IliasClient>>>>,
}

impl IliasTree {
    pub async fn new() -> Self {
        if let Ok(save_data) = read_to_string("path") {
            serde_json::from_str(&save_data).unwrap()
        } else {
        }
        Self {
            tree: read_to_string("path")
                .ok()
                .map(|data| Arc::new(Mutex::new(serde_json::from_str::<IlNode>(&data).unwrap())))
                .unwrap_or_default(),
            client: Arc::new(Mutex::new(IliasClient::new().await.ok().map(Arc::new))),
        }
    }

    pub async fn update_root(&self) -> Result<(), TreeError> {
        let client = self.client.lock().unwrap().take();
        if let Some(client) = client {
            update_node(client, self.tree.clone()).await.unwrap()?;
        }
        Ok(())
    }

    pub async fn login(&self, creds: Credentials) -> Result<(), ClientError> {
        let client = IliasClient::with_creds(creds).await?;
        *self.client.lock().unwrap() = Some(Arc::new(client));
        Ok(())
    }

    pub fn is_authenticated(&self) -> bool {
        self.client.lock().unwrap().is_some()
    }

    pub fn get_root_node(&self) -> IlNode {
        self.tree.lock().unwrap().clone()
    }

    pub fn save(&self) {
        let data = self.tree.lock().unwrap();
        fs::write("save.json", serde_json::to_string(&*data).unwrap()).unwrap()
    }
}
