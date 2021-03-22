use std::{
    path::PathBuf,
    sync::Arc,
};
use rocket::{response::NamedFile, State};
use rocket_contrib::json::{json, Json, JsonValue};
use tokio::fs::{read, read_to_string};

use crate::tree::{ILiasTree, IlNode};
use crate::client::ClientError;

#[get("/api/node")]
pub fn get_node(node: State<Arc<ILiasTree>>) -> Json<IlNode> {
    let node = node.get_root_node().unwrap().lock().unwrap();
    Json(node.clone())
}

#[get("/api/update")]
pub async fn update(node: State<'_, Arc<ILiasTree>>) -> JsonValue {
    if let Err(err) = node.update_ilias().await{
        
        match err {
            ClientError::NoToken => {
                if let Ok(token) = read_to_string("token.txt").await {
                    node.set_client_token(&token);
                }
                return json!({"status": "set_token"})
            }
            _ => {return json!({"status": format!("{:?}",err)})}
        }
        //
    }
    let node = node.get_root_node().unwrap().lock().unwrap();
    //Json(ReturnType::Ok(node.clone()))
    json!({"status": "ok", "node": &*node})
}

#[get("/")]
pub async fn index() -> std::result::Result<NamedFile, std::io::Error> {
    NamedFile::open("C:/dev/repositories/BettIlias/Frontend/dist/index.html").await
}

#[get("/api/open/<file..>")]
pub fn open_file(file: PathBuf) -> std::result::Result<(), std::io::Error> {
    open::that(file)?;
    Ok(())
}
