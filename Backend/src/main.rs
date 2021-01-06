#![feature(proc_macro_hygiene, decl_macro)]
use hyper::Client;
use hyper_tls::HttpsConnector;
use log::info;
use rocket::State;
use rocket_contrib::json::Json;
use std::sync::{Arc, Mutex};
use sync::{FileSelect, FileWatcher};
use tree::{IlNode, get_or_create_ilias_tree};
#[macro_use] extern crate rocket;

mod config;
mod helpers;
mod sync;
mod tree;

pub type IdSize = u16;

#[get("/")]
fn index(node: State<Arc<Mutex<IlNode>>>) -> Json<IlNode> {
    let node = node.lock().unwrap();
    Json(node.clone())
}

#[tokio::main]
async fn main(){
    env_logger::init();
    let https = HttpsConnector::new();
    let client = Arc::new(Client::builder().build::<_, hyper::Body>(https));

    let mut file_watcher = FileWatcher::new();

    let ilias_tree = get_or_create_ilias_tree(client.clone(), &mut file_watcher).await.unwrap();
    
    info!("sync structure to local filessystem");

    //sync::sync(ilias_tree.clone(), client.clone()).await?;

    info!("sync files");
    //add_to_file_watcher(&ilias_tree.lock().unwrap(), &mut file_watcher, "Bischte Dumm".to_string()); //remove
    /* file_watcher
        .sync(ilias_tree, FileSelect::All, client.clone())
        .await?;  */
   
    rocket::ignite().mount("/", routes![index]).manage(ilias_tree.clone()).launch();
}
