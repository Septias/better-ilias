use crate::{config::Config, tree::IlNode, IdSize};
use hyper::{
    body::HttpBody as _, client::HttpConnector, Body, Client, Method, Request, StatusCode,
};
use hyper_tls::HttpsConnector;
use log::error;
use scraper::Html;
use std::sync::{Arc, Mutex};



pub fn get_node(node: Arc<Mutex<IlNode>>, id: IdSize) -> Option<Arc<Mutex<IlNode>>> {
    let tree = node.lock().unwrap();
    if tree.id == id {
        return Some(node.clone());
    }
    if let Some(children) = &tree.children {
        //let mut children_iter = children.iter();
        let mut smallest = 0; //children_iter.next().unwrap().id;
        for (index, child) in children.iter().enumerate() {
            if child.lock().unwrap().id <= id {
                smallest = index as IdSize
            } else {
                break;
            }
        }
        get_node(children[smallest as usize].clone(), id)
    } else {
        None
    }
}
