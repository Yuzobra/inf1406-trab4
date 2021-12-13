use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use crate::external_communicator::ExternalConnectionRequest;
use crate::external_communicator::ExternalConnectionRequestType;
use crate::utils::RequestInfo;

#[derive(Deserialize, Debug, Clone)]
pub struct HeartbeatInfo {
    pub server_number: i32,
    pub timestamp: String,
}

pub struct ValueTableRequest {
    pub heartbeat_info: HeartbeatInfo,
}

pub enum ValueTableRequestType {
    Insert,
    Search,
}

pub fn create_heartbeat_handler() -> Sender<ValueTableRequest> {
    let (tx, rx) = channel();

    std::thread::spawn(move || {
        _run_main_loop(rx);
    });

    return tx.clone();
}

fn _run_main_loop(rx: Receiver<ValueTableRequest>) {
    let mut last_heartbeats: HashMap<i32, i32> = HashMap::new();

    for received in rx {
        if let ValueTableRequestType::Insert = received.request_type {
            last_heartbeats.insert(
                received.request_info.key.clone(),
                received.request_info.value,
            );
        }
    }
}
