use std::path::PathBuf;

use rocket::{response::NamedFile, State};
use rocket_contrib::{json::Json, serve::StaticFiles};

#[get("/api/node")]
pub fn api(node: State<Arc<Mutex<IlNode>>>) -> Json<IlNode> {
    let node = node.lock().unwrap();
    Json(node.clone())
}

#[get("/")]
pub fn index() -> std::result::Result<NamedFile, std::io::Error> {
    NamedFile::open("C:/dev/repositories/BettIlias/Frontend/dist/index.html")
}

#[get("/api/open/<file..>")]
pub fn open_file(file: PathBuf) -> std::result::Result<(), std::io::Error> {
    open::that(file)?;
    Ok(())
}
