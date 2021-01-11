#![feature(proc_macro_hygiene, decl_macro)]
use hyper::Client;
use hyper_tls::HttpsConnector;
use log::{error, info};
use rocket::{http::RawStr, response::NamedFile, State};
use rocket_contrib::{json::Json, serve::StaticFiles};
use ron::ser::{to_string_pretty, PrettyConfig};
use std::{
    env::current_dir,
    path::PathBuf,
    process::ExitStatus,
    sync::{Arc, Mutex},
};
use sync::{FileSelect, FileWatcher};
use tokio::{fs::File, io::AsyncWriteExt};
use tree::{get_or_create_ilias_tree, IlNode};
#[macro_use]
extern crate rocket;
use open;

mod config;
mod helpers;
mod sync;
mod tree;

pub type IdSize = u16;

#[get("/api/node")]
fn api(node: State<Arc<Mutex<IlNode>>>) -> Json<IlNode> {
    let node = node.lock().unwrap();
    Json(node.clone())
}

#[get("/")]
fn index() -> std::result::Result<NamedFile, std::io::Error> {
    NamedFile::open("C:/dev/repositories/BettIlias/Frontend/dist/index.html")
}

#[get("/api/open/<file..>")]
fn open_file(file: PathBuf) -> std::result::Result<(), std::io::Error> {
    open::that(file)?;
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let https = HttpsConnector::new();
    let client = Arc::new(Client::builder().build::<_, hyper::Body>(https));

    let mut file_watcher = FileWatcher::new().await;

    let ilias_tree = get_or_create_ilias_tree(client.clone(), &mut file_watcher)
        .await
        .unwrap();

    info!("sync structure to local filessystem");
    sync::sync(ilias_tree.clone(), client.clone())
        .await
        .unwrap();

    info!("sync files");

    file_watcher
        .sync(ilias_tree.clone(), FileSelect::All, client.clone())
        .await
        .unwrap();

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
    rocket::ignite()
        .mount(
            "/assets/",
            StaticFiles::from("C:/dev/repositories/BettIlias/Frontend/dist/assets"),
        )
        .mount("/", routes![api, index, open_file])
        .manage(ilias_tree.clone())
        .launch();
}
