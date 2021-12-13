use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use crate::external_communicator::ExternalConnectionRequest;
use crate::external_communicator::ExternalConnectionRequestType;
use crate::utils::RequestInfo;

pub struct ValueTableRequest {
    pub request_type: ValueTableRequestType,
    pub request_info: RequestInfo,
}

pub enum ValueTableRequestType {
    Insert,
    Search,
}

pub fn create_valuetable_handler(
    external_communicator_tx: Sender<ExternalConnectionRequest>,
) -> Sender<ValueTableRequest> {
    let (tx, rx) = channel();

    std::thread::spawn(move || {
        _run_main_loop(rx, external_communicator_tx);
    });

    return tx.clone();
}

fn _run_main_loop(
    rx: Receiver<ValueTableRequest>,
    external_communicator_tx: Sender<ExternalConnectionRequest>,
) {
    let mut node_values = HashMap::new();

    for received in rx {
        if let ValueTableRequestType::Insert = received.request_type {
            node_values.insert(
                received.request_info.key.clone(),
                received.request_info.value,
            );
        } else if let ValueTableRequestType::Search = received.request_type {
            if let Some(value) = node_values.get(received.request_info.key.as_str()) {
                println!("Found saved info: {}", value);

                let external_request = ExternalConnectionRequest {
                    request_type: ExternalConnectionRequestType::SearchReturn,
                    value: *value,
                    return_topic: received.request_info.return_topic,
                };
                external_communicator_tx.send(external_request).unwrap(); // TODO: Exception handle?
            } else {
                println!(
                    "Unable to find saved info for key {}",
                    received.request_info.key
                );
            }
        }
    }
}
