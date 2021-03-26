#![feature(proc_macro_hygiene, decl_macro, async_closure)]
//#![windows_subsystem = "windows"]

use itertools::Itertools;
use lazy_static::lazy_static;
use log::error;
use rocket_contrib::serve::StaticFiles;
use std::{
    env::Args,
    path::{Path, PathBuf},
    sync::Arc,
};
#[macro_use]
extern crate rocket;
use tokio::task::JoinHandle;
use tree::ILiasTree;

mod client;
mod server;
mod tree;

pub type IdSize = u16;

fn get_path(mut args: Args) -> Result<PathBuf, String> {
    let save_string = args.nth(1).unwrap();
    let parts = save_string.split(|a| a == '=').collect_vec();

    if parts.len() < 2 {
        return Err(String::from(
            " Argument must be of the form save_path=\"<path>\" ",
        ));
    }
    if parts[0] != String::from("save_path") {
        return Err(String::from(
            " Argument must be of the form save_path=\"<path>\"",
        ));
    }
    Ok(PathBuf::from(parts[1]))
}

lazy_static! {
    pub static ref FRONTEND_BASE_PATH: PathBuf = {
        #[cfg(debug_assertions)]
        return PathBuf::from("../Frontend/");

        #[cfg(not(debug_assertions))]
        return PathBuf::from("./");
    };
    pub static ref BACKEND_BASE_PATH: PathBuf = {
        let args = std::env::args();
        if args.len() > 1 {
            get_path(args).unwrap_or_else(|err| {
                error!("{}", err);
                PathBuf::from("./")
            })
        } else {
            if cfg!(debug_assertions) {
                PathBuf::from("./data/")
            } else {
                PathBuf::from("./")
            }
        }
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    let ilias = Arc::new(ILiasTree::from_file("structure.ron".into()).await?);

    let ilias_clone = ilias.clone();
    let _: JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
        ilias_clone.download_files().await?;
        Ok(())
    });

    #[cfg(not(debug_assertions))]
    if open::that("http://localhost:2020").is_err() {
        error!("couldn't open browser");
    }

    rocket::ignite()
        .mount(
            "/assets/",
            StaticFiles::from(FRONTEND_BASE_PATH.join("dist/assets")),
        )
        .mount(
            "/",
            routes![
                server::get_node,
                server::index,
                server::open_file,
                server::update,
                server::set_credentials,
                server::favicon
            ],
        )
        .manage(ilias)
        .launch()
        .await?;

    Ok(())
}
