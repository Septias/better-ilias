#![feature(proc_macro_hygiene, decl_macro, async_closure)]

use futures::future::join;
use log::error;
use rocket_contrib::serve::StaticFiles;
use std::{sync::Arc, thread};
#[macro_use]
extern crate rocket;

use open;
use tokio::sync::mpsc;
use tree::ILiasTree;

mod client;
mod server;
mod tree;

pub type IdSize = u16;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    let (sender, receiver) = mpsc::unbounded_channel();

    let ilias = Arc::new(ILiasTree::from_file("structure.ron".into()).await?);

    let ilias_clone = ilias.clone();
    thread::spawn(move || {
        rocket::ignite()
            .mount(
                "/assets/",
                StaticFiles::from("C:/dev/repositories/BettIlias/Frontend/dist/assets"),
            )
            .mount("/", routes![server::api, server::index, server::open_file])
            .manage(ilias_clone)
            .launch();
    });

    let ft1 = ilias.download_files(receiver);

    let ft2 = ilias.update_ilias(sender);

    let (res1, res2) = join(ft1, ft2).await;

    res1??;
    res2?;

   

    // open browser
    if open::that("http://localhost:2020").is_err() {
        error!("couldn't open browser");
    }

    Ok(())
}
