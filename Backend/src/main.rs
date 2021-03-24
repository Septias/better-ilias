#![feature(proc_macro_hygiene, decl_macro, async_closure)]

use log::error;
use rocket_contrib::serve::StaticFiles;
use std::sync::Arc;
#[macro_use]
extern crate rocket;


use tokio::task::JoinHandle;
use tree::ILiasTree;

mod client;
mod server;
mod tree;

pub type IdSize = u16;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    let ilias = Arc::new(ILiasTree::from_file("structure.ron".into()).await?);

    let ilias_clone = ilias.clone();
    let _: JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
        ilias_clone.download_files().await?;
        Ok(())
    });
    
    if open::that("http://localhost:2020").is_err() {
        error!("couldn't open browser");
    }

    rocket::ignite()
        .mount(
            "/assets/",
            StaticFiles::from("C:/dev/repositories/BettIlias/Frontend/dist/assets"),
        )
        .mount("/", routes![server::get_node, server::index, server::open_file, server::update, server::set_credentials])
        .manage(ilias)
        .launch().await?;

    Ok(())
}
