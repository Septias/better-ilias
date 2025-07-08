#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use client::{ClientError, Credentials};
use ilias::{IlNode, IliasTree};
use log::{info, warn};
use std::{path::PathBuf, sync::Arc};
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
async fn login_cached(ilias: tauri::State<'_, Arc<IliasTree>>) -> Result<(), String> {
    ilias.login_cached().await.map_err(|err| err.to_string())
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

#[tauri::command]
fn open(path: PathBuf) -> Result<(), String> {
    match open::that(&path) {
        Ok(_) => {
            info!("{path:?}");
            Ok(())
        }
        Err(e) => {
            warn!("{e:?}");
            Err(e.to_string())
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    env_logger::init();

    let tree = Arc::new(IliasTree::new().await);
    let tree_clone = tree.clone();
    let app = tauri::Builder::default()
        .manage(tree)
        .invoke_handler(tauri::generate_handler![
            login,
            login_cached,
            is_authenticated,
            update_root,
            get_root,
            open
        ])
        .build(generate_context!())
        .expect("error while running tauri application");

    app.run(move |_app_handle, e| {
        if let tauri::RunEvent::Exit { .. } = e {
            tree_clone.save().map_err(|err| warn!("{err}")).ok();
        }
    });
}
