#![feature(proc_macro_hygiene, decl_macro, async_closure)]

use futures::future::join;
use log::error;
use rocket_contrib::serve::StaticFiles;
use ron::ser::{to_string_pretty, PrettyConfig};
use std::sync::Arc;
#[macro_use]
extern crate rocket;

use open;
use tokio::{fs::File, io::AsyncWriteExt, sync::mpsc};
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

    let ft1 = ilias.download_files(receiver);
    let ft2 = ilias.update_ilias(sender);
    let (res1, res2) = join(ft1, ft2).await;
    res1??;
    res2?;

    let pretty = PrettyConfig::new()
        .with_separate_tuple_members(true)
        .with_enumerate_arrays(true);
    let mut writer = File::create("structure.ron")
        .await
        .expect("unable to create save-file");
    let s = to_string_pretty(&*ilias.get_root_node().unwrap().lock().unwrap(), pretty).unwrap();

    if writer.write_all(s.as_bytes()).await.is_err() {
        error!("Can't save structure.ron");
    }

    // open browser
    if open::that("http://localhost:2020").is_err() {
        error!("couldn't open browser");
    }

    let ilias_clone = ilias.clone();
    rocket::ignite()
        .mount(
            "/assets/",
            StaticFiles::from("C:/dev/repositories/BettIlias/Frontend/dist/assets"),
        )
        .mount("/", routes![server::get_node, server::index, server::open_file, server::update])
        .manage(ilias_clone)
        .launch();

    Ok(())
}
