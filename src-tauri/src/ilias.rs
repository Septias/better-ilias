use crate::{
    client::{ClientError, Credentials, IliasClient},
    tree::{update_root, TreeError},
};
use ::tauri::api::path::cache_dir;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::api::{file::read_string, path::home_dir};

pub const ILIAS_ROOT: &str =
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
            children: Some(vec![]),
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
        version: usize,
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

    pub fn is_file(&self) -> bool {
        matches!(self, IlNodeType::File { .. })
    }
}

#[derive(Clone)]
pub struct IliasTree {
    tree: WrappedNode,
    client: Arc<Mutex<Option<Arc<IliasClient>>>>,
}

fn saves_path() -> Option<PathBuf> {
    cache_dir().map(|mut path| {
        path.push("better-ilias/save.json");
        path
    })
}

pub(crate) fn root_path() -> Option<PathBuf> {
  home_dir().map(|mut path| {
    path.push("better-ilias");
    path
})
}

impl IliasTree {
    pub async fn new() -> Self {
        Self {
            tree: saves_path()
                .map(|path| read_string(path).ok())
                .flatten()
                .map(|data| Arc::new(Mutex::new(serde_json::from_str::<IlNode>(&data).unwrap())))
                .unwrap_or_default(),
            client: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn update_root(&self) -> Result<(), TreeError> {
        let client = self.client.lock().unwrap().clone();
        if let Some(client) = client {
            info!("updating root node");
            update_root(client.clone(), self.tree.clone())
                .await
                .unwrap()?;
            info!("successfully updated root node");
            Ok(())
        } else {
            Err(TreeError::Client(ClientError::NoToken))
        }
    }

    pub async fn login(&self, creds: Credentials) -> Result<(), ClientError> {
        match IliasClient::with_creds(creds).await {
            Ok(client) => *self.client.lock().unwrap() = Some(Arc::new(client)),
            Err(e) => {
                warn!("{e}");
                return Err(e);
            }
        }
        Ok(())
    }

    pub async fn login_cached(&self) -> anyhow::Result<()> {
        let client = IliasClient::new().await?;
        *self.client.lock().unwrap() = Some(Arc::new(client));
        Ok(())
    }

    pub fn is_authenticated(&self) -> bool {
        self.client.lock().unwrap().is_some()
    }

    pub fn get_root_node(&self) -> IlNode {
        self.tree.lock().unwrap().clone()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let data = self.tree.lock().unwrap();
        let path = saves_path().ok_or(anyhow::anyhow!("can't create path"))?;
        if let Err(e) = fs::write(&path, serde_json::to_string(&*data).unwrap()) {
            if e.kind() == io::ErrorKind::NotFound {
                fs::create_dir_all(path.parent().unwrap())?;
                fs::write(path, serde_json::to_string(&*data).unwrap())?;
            }
        }
        Ok(())
    }
}
