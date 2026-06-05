// Imports the info struct from message.rs
use crate::message::WorkerInfo;

use std::sync::{LazyLock, Mutex};

// Struct that stores the Node's name and ID
#[derive(Debug)]
pub struct NodeInfo {
    pub name: String,
    pub id: i32,
    pub info: WorkerInfo
}

pub static NODE_LIST: LazyLock<Mutex<Vec<NodeInfo>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn ProcessData(worker_info: WorkerInfo){
}

pub fn CreateNode(worker_info: &WorkerInfo){

    let mut list = NODE_LIST.lock().unwrap();

    list.push(NodeInfo { name: worker_info.hostname.clone(), id: 1, info: worker_info.clone() });


}

pub fn RemoveNode(worker_name: &String){ 

    let mut list = NODE_LIST.lock().unwrap();

    list.retain(|item| item.name != worker_name.clone());
    
}
