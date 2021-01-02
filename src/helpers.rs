use crate::{IdSize, config::Config, tree::IlNode};
use hyper::{
    body::HttpBody as _, client::HttpConnector, Body, Client, Method, Request, StatusCode,
};
use hyper_tls::HttpsConnector;
use log::error;
use scraper::Html;
use std::sync::Arc;

pub async fn request_il_page(
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
    if resp.status() != StatusCode::OK {
        error!(
            "{} Problem with requestion ilias-page \" {}\"",
            resp.status(),
            uri
        );
    }
    let mut bytes = vec![];
    while let Some(chunk) = resp.body_mut().data().await {
        let chunk = chunk?;
        bytes.extend(&chunk[..]);
    }
    Ok(Html::parse_document(std::str::from_utf8(&bytes)?))
}

pub fn get_node(tree: &IlNode, id: IdSize) -> Option<&IlNode> {
    if tree.id == id {
        return Some(tree)
    }
    if let Some(children) = &tree.children {
        //let mut children_iter = children.iter();
        let mut smallest = 0;//children_iter.next().unwrap().id;
        for (index, child) in children.iter().enumerate(){
            if child.id <= id {
                smallest = index as IdSize
            } else {
                break;
            }
        }
        get_node(&children[smallest as usize], id) 
    } else {
        None
    }
}
