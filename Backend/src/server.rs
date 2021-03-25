use rocket::{response::NamedFile, State};
use rocket_contrib::json::{json, Json, JsonValue};
use serde::Deserialize;
use std::{path::PathBuf, sync::Arc};
use tokio::fs;

use crate::client::ClientError;
use crate::tree::{ILiasTree, IlNode};
use crate::FRONTEND_BASE_PATH;


#[get("/api/node")]
pub fn get_node(node: State<Arc<ILiasTree>>) -> Json<IlNode> {
    let node = node.get_root_node().unwrap().lock().unwrap();
    Json(node.clone())
}

#[get("/api/update")]
pub async fn update(node: State<'_, Arc<ILiasTree>>) -> JsonValue {
    if let Err(err) = node.update_ilias().await {
        match err {
            ClientError::NoToken => return json!({"status": "set_token"}),
            _ => return json!({ "status": format!("{:?}", err) }),
        }
    }
    node.save().await;
    let node = node.get_root_node().unwrap().lock().unwrap();
    json!({"status": "ok", "node": &*node})
}

#[get("/")]
pub async fn index() -> std::result::Result<NamedFile, std::io::Error> {
    NamedFile::open(FRONTEND_BASE_PATH.join("dist/index.html")).await
}

#[get("/api/open/<file..>")]
pub fn open_file(file: PathBuf) -> std::result::Result<(), std::io::Error> {
    open::that(file)?;
    Ok(())
}

#[get("/favicon.ico")]
pub async fn favicon() -> Result<NamedFile, std::io::Error> {
    NamedFile::open(FRONTEND_BASE_PATH.join("dist/favicon-32x32.png")).await
}

#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
    persistent: bool,
}

#[post("/api/credentials", data = "<credentials>")]
pub async fn set_credentials(
    credentials: Json<Credentials>,
    node: State<'_, Arc<ILiasTree>>,
) -> JsonValue {
    let creds = [
        credentials.username.to_owned(),
        credentials.password.to_owned(),
    ];
    let err = node.client.acquire_token(&creds).await;

    if let Err(err) = err {
        json!({ "status": format!("{}", err) })
    } else {
        if credentials.persistent
            && fs::write("credentials.txt", creds.join("\n"))
                .await
                .is_err()
        {
            error!("couldn't write credentials to file");
        }

        json!({"status": "ok"})
    }
}
