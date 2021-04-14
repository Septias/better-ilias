use rocket::{response::NamedFile, State};
use rocket_contrib::json::{json, Json, JsonValue};
use serde::Deserialize;
use crate::client::ClientError;
use crate::schema::notes;
use crate::tree::{IlNode, IliasTree};
use diesel::{QueryDsl, RunQueryDsl};
use itertools::Itertools;
use lazy_static::lazy_static;
use rocket::{
    fairing::AdHoc,
    response::{Debug, NamedFile},
    State,
};
use rocket_contrib::{
    json::{json, Json, JsonValue},
    serve::StaticFiles,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{env::Args, path::PathBuf};
use tokio::fs;

use crate::client::ClientError;
use crate::tree::{ILiasTree, IlNode};
use crate::FRONTEND_BASE_PATH;

#[get("/api/node")]
pub fn get_node(node: State<Arc<ILiasTree>>) -> Json<IlNode> {
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
pub struct Note {
    pub uri: String,
    pub course: String,
    pub body: String,
}

#[database("sqlite_db")]
struct NotesDB(diesel::SqliteConnection);

#[get("/notes")]
async fn get_notes(conn: NotesDB) -> Result<Json<Vec<String>>> {
    let ids = conn
        .run(move |conn| notes::table.select(notes::course).load(conn))
        .await?;

    Ok(Json(ids))
}
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

#[derive(Deserialize)]
pub struct File {
    path: String,
}

#[post("/api/open", data = "<file>")]
pub fn open_file(file: Json<File>) -> std::result::Result<(), std::io::Error> {
    open::that(&file.path)?;
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

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel SQLite Stage", |rocket| async {
        rocket
            .attach(NotesDB::fairing())
            //.attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
            .mount("/", routes![index])
            .mount(
                "/api",
                routes![
                    get_node,
                    get_notes,
                    open_file,
                    update,
                    set_credentials,
                    favicon
                ],
            )
            .mount(
                "/assets/",
                StaticFiles::from(FRONTEND_BASE_PATH.join("dist/assets")),
            )
    })
}
