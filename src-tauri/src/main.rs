#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

use client::{ClientError, Credentials};
use ilias::{IlNode, IliasTree};
use log::warn;
use tauri::generate_context;
use tree::TreeError;
mod client;
mod ilias;
mod string_serializer;
mod tree;

#[tauri::command]
async fn login(
    ilias: tauri::State<'_, Arc<IliasTree>>,
    creds: Credentials,
) -> Result<(), ClientError> {
    ilias.login(creds).await
}

#[tauri::command]
fn is_authenticated(ilias: tauri::State<'_, Arc<IliasTree>>) -> bool {
    ilias.is_authenticated()
}

#[tauri::command]
async fn update_root(ilias: tauri::State<'_, Arc<IliasTree>>) -> Result<(), TreeError> {
    ilias.update_root().await
}

#[tauri::command]
fn get_root(ilias: tauri::State<'_, Arc<IliasTree>>) -> IlNode {
    ilias.get_root_node()
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let tree = Arc::new(IliasTree::new().await);
    let tree_clone = tree.clone();
    let app = tauri::Builder::default()
        .manage(tree)
        .invoke_handler(tauri::generate_handler![
            login,
            is_authenticated,
            update_root,
            get_root
        ])
        .build(generate_context!())
        .expect("error while running tauri application");

    app.run(move |_app_handle, e| {
        if let tauri::RunEvent::Exit { .. } = e {
            tree_clone.save().map_err(|err| warn!("{err}")).ok();
        }
    });
}
