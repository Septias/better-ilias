use crate::{
    client::{Credentials, IliasClient},
    tree::update_node,
};
use futures::{SinkExt, StreamExt};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, read_to_string},
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{
    net::{TcpListener, TcpStream},
    signal,
};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result},
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

#[derive(Serialize, Deserialize, Clone)]
pub struct IliasTree {
    tree: Option<IlNode>,
}

impl IliasTree {
    pub async fn new() -> Self {
        if let Ok(save_data) = read_to_string("path") {
            serde_json::from_str(&save_data).unwrap()
        } else {
            IliasTree {
                tree: Some(IlNode::default()),
            }
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let addr = "localhost:2654";
        let tcp_listener = TcpListener::bind(addr).await.unwrap();
        info!("listening on {addr}");

        let Self { mut tree } = self;
        let root_node = Arc::new(Mutex::new(tree.take().unwrap().clone()));
        {
            let root_node = root_node.clone();
            tokio::spawn(async move {
                while let Ok((stream, _)) = tcp_listener.accept().await {
                    let peer = stream
                        .peer_addr()
                        .expect("connected streams should have a peer address");
                    {
                        info!("New connection: {}", peer);
                        let root_node = root_node.clone();
                        tokio::spawn(async move { new_client(stream, root_node).await.unwrap() });
                    }
                }
            });
        }
        signal::ctrl_c().await.unwrap();
        Self::save(&root_node).await;
        Ok(())
    }

    pub async fn save(data: &WrappedNode) {
        fs::write("save.json", serde_json::to_string(&data).unwrap()).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
enum Request {
    Update,
    Login(Credentials),
}

#[derive(Serialize, Deserialize)]
enum Response {
    Updated,
    Success,
    NotAuthenticated,
}

impl Response {
    fn into_resp(&self) -> Message {
        serde_json::to_string(&self).unwrap().into()
    }
}

async fn new_client(stream: TcpStream, root_node: WrappedNode) -> anyhow::Result<()> {
    let mut stream = accept_async(stream).await.expect("Failed to accept");
    let mut client = IliasClient::new().await.ok().map(Arc::new);

    while let Some(msg) = stream.next().await {
        if let Ok(Message::Text(text)) = msg {
            let rq: Request = serde_json::from_str(&text)?;
            let resp = match rq {
                Request::Update => {
                    if let Some(client) = &client {
                        update_node(client.clone(), root_node.clone()).await??;
                        Response::Updated.into_resp()
                    } else {
                        Response::NotAuthenticated.into_resp()
                    }
                }
                Request::Login(creds) => {
                    if let Ok(new_client) = IliasClient::with_creds(creds).await {
                        client = Some(Arc::new(new_client));
                        Response::Success.into_resp()
                    } else {
                        Response::NotAuthenticated.into_resp()
                    }
                }
            };
            stream.send(resp).await.unwrap();
        }
    }
    Ok(())
}
