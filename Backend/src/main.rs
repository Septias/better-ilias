#![feature(proc_macro_hygiene, decl_macro, async_closure)]
//#![windows_subsystem = "windows"]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_contrib;

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

    #[cfg(not(debug_assertions))]
    if open::that("http://localhost:2020").is_err() {
        error!("couldn't open browser");
    }

    rocket::build()
        .attach(server::stage())
        .manage(ilias)
        .launch()
        .await?;

    Ok(())
}
