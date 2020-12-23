use hyper::{body::HttpBody as _, client::HttpConnector, Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use log::{error, info};
use ron::{
    de::from_reader,
    ser::{to_string_pretty, to_writer_pretty, PrettyConfig},
};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write};
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    // I hope this is the right way to do it
    let client = Box::leak(Box::new(
        Client::builder().build::<_, hyper::Body>(HttpsConnector::new()),
    ));

    let ilias_tree: IlNode = if let Some(iliasTree) = match File::open("structure.ron") {
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
        iliasTree
    } else {
        info!("fetching ilias_tree");
        let mut ilias_tree = load_ilias(
            "ilias.php?ref_id=1836117&cmdClass=ilrepositorygui&cmdNode=yj&baseClass=ilrepositorygui&cmd=view"
                .to_string(),
            "Rechnernetze".to_string(),
            client,
        )
        .await?.expect("couldn't fetch Ilias-tree");

        set_ids(&mut ilias_tree, &mut 0);

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

    //println!("{:#?}", ilias_tree);
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

async fn request_il_page(
    uri: &str,
    client: &Client<HttpsConnector<HttpConnector>>,
) -> Result<Html, Box<dyn std::error::Error + Send + Sync>> {
    let req = Request::builder()
        .method(Method::GET)
        .uri("https://ilias.uni-freiburg.de/".to_owned() + &uri)
        .header("authority", "ilias.uni-freiburg.de")
        .header("upgrade-insecure-requests", 1)
        .header("dnt", 1)
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.88 Safari/537.36")
        .header("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9")
        .header("sec-fetch-site", "same-origin")
        .header("sec-fetch-mode", "navigate")
        .header("sec-fetch-user", "?1")
        .header("sec-fetch-dest", "document")
        .header("referer", "https://ilias.uni-freiburg.de/ilias.php?baseClass=ilPersonalDesktopGUI&cmd=jumpToSelectedItems")
        .header("accept-language", "de-DE,de;q=0.9,en-US;q=0.8,en;q=0.7")
        .header("cookie", "iom_consent=00000000000000&1604408733754; ioam2018=000ef5c0cec6382585fa1559e:1632834334057:1604408734057:.uni-freiburg.de:6:ak025:dbs:noevent:1604871141009:nb8xt2; ilClientId=unifreiburg; _shibsession_64656661756c7468747470733a2f2f696c6961732e756e692d66726569627572672e64652f73686962626f6c657468=_0ed19f012091e81c7f66970654a4def0; PHPSESSID=qrb2h55lg6hh17cn9ckmnpiid0")
        .body(Body::empty()).unwrap();

    let mut resp = client.request(req).await?;
    let mut bytes = vec![];
    while let Some(chunk) = resp.body_mut().data().await {
        let chunk = chunk?;
        bytes.extend(&chunk[..]);
    }
    Ok(Html::parse_document(std::str::from_utf8(&bytes)?))
}

async fn get_child_pages(
    uri: &str,
    client: &Client<HttpsConnector<HttpConnector>>,
) -> Vec<(String, String)> {
    let containers = Selector::parse(".ilContainerListItemOuter .il_ContainerItemTitle a").unwrap();
    let html = request_il_page(uri, client).await.unwrap();
    let elements = html.select(&containers);
    let mut element_infos = vec![];
    for element in elements {
        element_infos.push((
            element.value().attr("href").unwrap().to_string(),
            element.inner_html(),
        ))
    }
    element_infos
}

#[derive(Debug, Deserialize, Serialize)]
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

fn load_ilias(
    uri: String,
    title: String,
    client: &'static Client<HttpsConnector<HttpConnector>>,
) -> tokio::task::JoinHandle<Option<IlNode>> {
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
                    let child_elements = get_child_pages(&uri, client).await;
                    // create children
                    let mut handles = vec![];
                    for element in child_elements {
                        handles.push(load_ilias(element.0, element.1, &client));
                    }
                    // load children and add them to the node
                    let mut children = vec![];
                    for handle in handles {
                        if let Ok(Some(child)) = handle.await {
                            children.push(child);
                        }
                    }
                    node.children = Some(children);
                }
                IlNodeType::File => {
                    node.breed = IlNodeType::File;
                    node.sync = true;
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
