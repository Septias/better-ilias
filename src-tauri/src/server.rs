use crate::client::ClientError;
use crate::schema::notes;
use crate::tree::{IlNode, IliasTree};
use diesel::{ExpressionMethods, RunQueryDsl};
use itertools::Itertools;
use lazy_static::lazy_static;
use rocket::{
    fairing::AdHoc,
    response::{status::Created, Debug},
    Build, Rocket, State,
    serde::{json::{json, Json, Value}}
};
use rocket_sync_db_pools::database;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{env::Args, path::PathBuf};
use tokio::fs;

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

fn get_path(mut args: Args) -> Result<PathBuf, String> {
    let save_string = args.nth(1).unwrap();
    let parts = save_string.split(|a| a == '=').collect_vec();

    if parts.len() < 2 || parts[0] != String::from("save_path") {
        return Err(String::from(
            " Argument must be of the form save_path=\"<path>\" ",
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

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Insertable, Identifiable, AsChangeset,
)]
#[primary_key(uri)]
pub struct Note {
    pub uri: String,
    pub course: String,
    pub body: String,
}

#[database("sqlite_db")]
struct NotesDB(diesel::SqliteConnection);

#[get("/notes/list")]
async fn get_notes(conn: NotesDB) -> Result<Json<Vec<Note>>> {
    let ids = conn.run(move |conn| notes::table.load(conn)).await?;

    Ok(Json(ids))
}

#[post("/notes/create", data = "<note>")]
async fn create_note(db: NotesDB, note: Json<Note>) -> Result<Created<Json<Note>>> {
    let note_value = note.clone();
    db.run(move |conn| {
        diesel::insert_into(notes::table)
            .values(&*note_value)
            .execute(conn)
    })
    .await?;

    Ok(Created::new("/").body(note))
}

#[post("/notes/update", data = "<note>")]
async fn update_note(db: NotesDB, note: Json<Note>) -> Result<&'static str> {
    db.run(move |conn| {
        diesel::update(notes::table)
            .filter(notes::dsl::uri.eq(&note.uri))
            .set(&*note)
            .execute(conn)
    })
    .await?;

    Ok("ok")
}

#[get("/node")]
pub fn get_node(node: &State<Arc<IliasTree>>) -> Json<IlNode> {
    let node = node.get_root_node().unwrap().lock().unwrap();
    Json(node.clone())
}

#[get("/update")]
pub async fn update(node: &State<Arc<IliasTree>>) -> Value {
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

#[derive(Deserialize)]
pub struct File {
    path: String,
}

#[post("/open", data = "<file>")]
pub fn open_file(file: Json<File>) -> std::result::Result<(), std::io::Error> {
    open::that(&file.path)?;
    Ok(())
}


#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
    persistent: bool,
}

#[post("/credentials", data = "<credentials>")]
pub async fn set_credentials(
    credentials: Json<Credentials>,
    node: &State<Arc<IliasTree>>,
) -> Value {
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

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    // This macro from `diesel_migrations` defines an `embedded_migrations`
    // module containing a function named `run` that runs the migrations in the
    // specified directory, initializing the database.
    embed_migrations!("./migrations");

    let conn = NotesDB::get_one(&rocket)
        .await
        .expect("database connection");
    conn.run(|c| embedded_migrations::run(c))
        .await
        .expect("diesel migrations");

    rocket
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel SQLite Stage", |rocket| async {
        rocket
            .attach(NotesDB::fairing())
            .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
            //.attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
            .mount(
                "/api",
                routes![
                    get_node,
                    get_notes,
                    open_file,
                    update,
                    set_credentials,
                    create_note,
                    update_note
                ],
            )
    })
}
