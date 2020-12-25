use chrono::{DateTime, TimeZone, Utc};
use hyper::{Body, Client, Method, Request, StatusCode, body::HttpBody as _, client::HttpConnector};
use hyper_tls::HttpsConnector;
use log::{error, info};
use ron::{
    de::from_reader,
    ser::{to_string_pretty, PrettyConfig},
};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::{ErrorKind, Write}, path::PathBuf, sync::Arc, unimplemented};
use tokio::{fs::create_dir, task::{self, JoinHandle}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    let client = Arc::new(Client::builder().build::<_, hyper::Body>(HttpsConnector::new()));

    let ilias_tree: IlNode = if let Some(ilias_tree) = match File::open("structure.ron") {
        Ok(save) => {
            if let Ok(ilias_tree) = from_reader(save) {
                Some(ilias_tree)
            } else {
                None
            }
        }
        Err(_) => None,
    } {
        info!("loaded ilias_tree from file");
        ilias_tree
    } else {
        info!("fetching ilias_tree");
        let mut ilias_tree = get_ilias_tree(
            "ilias.php?ref_id=1836117&cmdClass=ilrepositorygui&cmdNode=yj&baseClass=ilrepositorygui&cmd=view"
                .to_string(),
            "Rechnernetze".to_string(),
            client.clone(),
        )
        .await?.expect("couldn't fetch Ilias-tree");

        set_ids(&mut ilias_tree, &mut 0);

        // save to file
        let pretty = PrettyConfig::new()
            .with_separate_tuple_members(true)
            .with_enumerate_arrays(true);
        let mut writer = File::create("structure.ron").expect("unable to create save-file");
        let s = to_string_pretty(&ilias_tree, pretty).unwrap();
        let write_result = writer.write_all(s.as_bytes());
        if let Err(_) = write_result {
            error!("Can't save structure.ron");
        }
        ilias_tree
    };
    // don't fkn drop ilias pls
    let ilias_tree = Box::leak(Box::new(ilias_tree));
    
    // sync ilias_tree to local files
    sync(ilias_tree, PathBuf::new(), client.clone()).await?;
    Ok(())
}

fn set_ids(node: &mut IlNode, id: &mut u16) {
    node.id = id.clone();
    *id += 1;
    if let Some(children) = node.children.as_mut() {
        for child in children.iter_mut() {
            set_ids(child, id);
        }
    }
}

struct Config();

impl Config{
    fn get_token() -> &'static str {
        "cl0bpgc6d4fkhonr8ok84tqvm3"
    }
}

struct FileWatcher {
    files: HashMap<String, DateTime<Utc>>
}

impl FileWatcher {
    async fn add_file(&mut self, uri: &str, last_changed: &str, path: &str, file_ending: &str){
        self.files.insert(uri.to_string(), Self::date_from_str(last_changed));
        FileWatcher::download_file(uri, path, file_ending).await;
    }
    async fn download_file(uri: &str, path: &str, file_ending: &str){

    }

    fn date_from_str(date_time_str: &str) -> DateTime<Utc>{
        Utc.ymd(2020, 12, 1).and_hms(4, 20, 00)
    }
    fn process(&self, uri: &str, date: &str){
        let last_changed = self.files.get(uri);
        match last_changed {
            Some(DateTime) => {}
            None => {}
        }
        unimplemented!();
    }
}

fn sync(
    node: &'static IlNode,
    mut path: PathBuf,
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut handles = vec![];
        match node.breed {
            IlNodeType::Folder => {
                if node.sync {
                    
                }

                path.push(&node.title);
                match create_dir(&path).await {
                    Ok(_) => {
                        info!("created Folder {}", &node.title)
                    }
                    Err(err) if err.kind() == ErrorKind::AlreadyExists => {}
                    Err(err) => {
                        error!("couldn't create folder \"{}\" - {}", &node.title, err)
                    }
                }

                if let Some(children) = &node.children {
                    for child in children
                        .iter()
                        .filter(|child| child.breed == IlNodeType::Folder)
                    {
                        handles.push(sync(child, path.clone(), client.clone()));
                    }
                }
            }
            _ => (),
        }
        for handle in handles {
            handle.await.unwrap();
        }
    })
}

async fn request_il_page(
    uri: &str,
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
) -> Result<Html, Box<dyn std::error::Error + Send + Sync>> {
    let req = Request::builder()
        .method(Method::GET)
        .uri("https://ilias.uni-freiburg.de/".to_owned() + &uri)
        .header("cookie", "PHPSESSID=".to_owned() + Config::get_token())
        .body(Body::empty())
        .unwrap();
    let mut resp = client.request(req).await?;
    if resp.status() != StatusCode::OK{
        error!("{} Problem with requestion ilias-page \" {}\"", resp.status(), uri);
    }
    let mut bytes = vec![];
    while let Some(chunk) = resp.body_mut().data().await {
        let chunk = chunk?;
        bytes.extend(&chunk[..]);
    }
    Ok(Html::parse_document(std::str::from_utf8(&bytes)?))
}

struct PageInfo {
    title: String,
    uri: String,
}

async fn get_child_pages(
    uri: &str,
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
) -> Vec<PageInfo> {

    let containers = Selector::parse(".ilContainerListItemOuter .il_ContainerItemTitle a").unwrap();
    let html = request_il_page(uri, client).await.unwrap();
    let elements = html.select(&containers);
    let mut element_infos = vec![];
    for element in elements {
        element_infos.push(PageInfo{
            uri: element.value().attr("href").unwrap().to_string(),
            title: element.inner_html().replace("/", " "),
        })
    }
    element_infos
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
enum IlNodeType {
    Forum,
    Folder,
    DirectLink,
    File,
}

fn get_il_node_type(uri: &str) -> Option<IlNodeType> {
    let cmd = uri
        .split("&")
        .find_map(|urlpiece| urlpiece.strip_prefix("cmd="));
    match cmd {
        Some("view") => Some(IlNodeType::Folder),
        Some("showThreads") => Some(IlNodeType::Forum),
        Some("calldirectlink") => Some(IlNodeType::DirectLink),
        Some(_) => None,
        None => {
            if uri.contains("goto.php") {
                Some(IlNodeType::File)
            } else {
                None
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct IlNode {
    title: String,
    id: u16,
    uri: String,
    sync: bool, // should this node be synced
    breed: IlNodeType,
    children: Option<Vec<IlNode>>,
}

fn get_ilias_tree(
    uri: String,
    title: String,
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
) -> JoinHandle<Option<IlNode>> {
    task::spawn(async move {
        if let Some(node_type) = get_il_node_type(&uri) {
            let mut node = IlNode {
                title,
                children: None,
                sync: false,
                breed: IlNodeType::File,
                uri: uri.clone(),
                id: 0,
            };
            match node_type {
                IlNodeType::Forum => {
                    node.breed = IlNodeType::Forum;
                }
                IlNodeType::Folder => {
                    node.breed = IlNodeType::Folder;
                    let child_elements = get_child_pages(&uri, client.clone()).await;
                    // create children
                    let mut handles = vec![];
                    for element in child_elements {
                        handles.push(get_ilias_tree(element.uri, element.title, client.clone()));
                    }
                    // load children and add them to the node
                    let mut children = vec![];
                    for handle in handles {
                        if let Ok(Some(child)) = handle.await {
                            if &child.breed == &IlNodeType::File {
                                node.sync = true
                            }
                            children.push(child);
                        }
                    }
                    node.children = Some(children);
                }
                IlNodeType::File => {
                    node.breed = IlNodeType::File;
                }
                IlNodeType::DirectLink => {
                    node.breed = IlNodeType::DirectLink;
                }
            };
            Some(node)
        } else {
            None
        }
    })
}
