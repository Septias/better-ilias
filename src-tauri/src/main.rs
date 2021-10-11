#![feature(proc_macro_hygiene, decl_macro, async_closure)]
#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::sync::Arc;
use tokio::task::JoinHandle;
use tree::IliasTree;

mod client;
mod schema;
mod server;
mod tree;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  env_logger::init();

  let ilias = Arc::new(IliasTree::from_file("structure.ron".into()).await?);

  let ilias_clone = ilias.clone();
  let _: JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
    ilias_clone.download_files().await?;
    Ok(())
  });

  tokio::spawn( async {
      rocket::build()
        .attach(server::stage())
        .manage(ilias)
        .launch()
        .await.unwrap();
  });

tauri::Builder::default()
    .run(tauri::generate_context!())
    .expect("error while running tauri application");

  Ok(())
}
