use futures::future::join_all;
use hyper::{body::HttpBody, client::HttpConnector, Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use log::{error, info, warn};
use ron::{de::from_bytes, to_string};
use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::ErrorKind,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{
    fs::{create_dir, File},
    io::{self, AsyncReadExt, AsyncWriteExt},
    task::{self, JoinHandle},
};

use crate::tree::{IlNode, IlNodeType};

pub fn sync(
    node: Arc<Mutex<IlNode>>,

)   {
    
    let mut sync_handles = vec![];
    let mut file_handles = vec![];
    {
        let node = node.lock().unwrap();
        match &node.breed {
            IlNodeType::Folder { sync, store_files, path } => {
                
                if let Some(children) = &node.children {
                    for child in children
                        .iter()
                        .filter(|child| child.lock().unwrap().breed == IlNodeType::Folder)
                    {   
                        sync_handles.push(tokio::spawn(async move {
                            sync(child.clone(), client.clone());
                        }));
                    }
                }
                
            }
            _ => ()
        };
    }

    for file_handle in file_handles {
        match file_handle.await {
            Ok(_) => {
                info!("created Folder ") //info!("created Folder {}", &node.title);
            }
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {}
            Err(_err) => {
                error!("couldn't create folder") //error!("couldn't create folder \"{}\" - {}", &node.title, err);
            }
        }
    }
    join_all(sync_handles).await;
    
}
