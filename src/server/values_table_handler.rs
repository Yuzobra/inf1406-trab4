use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use crate::external_communicator::ExternalConnectionRequest;
use crate::external_communicator::ExternalConnectionRequestType;
use crate::utils::ContentDumpInfo;
use crate::utils::RequestInfo;

pub struct ValueTableRequest {
    pub request_type: ValueTableRequestType,
    pub request_info: RequestInfo,
    pub server_num: i32,
}

pub enum ValueTableRequestType {
    Insert,
    Search,
    DumpContents,
}

pub fn create_valuetable_handler(
    node_values: HashMap<String, i32>,
    external_communicator_tx: Sender<ExternalConnectionRequest>,
) -> Sender<ValueTableRequest> {
    let (tx, rx) = channel();

    std::thread::spawn(move || {
        _run_main_loop(node_values, rx, external_communicator_tx);
    });

    return tx.clone();
}

fn _run_main_loop(
    mut node_values: HashMap<String, i32>,
    rx: Receiver<ValueTableRequest>,
    external_communicator_tx: Sender<ExternalConnectionRequest>,
) {
    for received in rx {
        if let ValueTableRequestType::Insert = received.request_type {
            node_values.insert(
                received.request_info.key.clone(),
                received.request_info.value,
            );
        } else if let ValueTableRequestType::Search = received.request_type {
            if let Some(value) = node_values.get(received.request_info.key.as_str()) {
                println!(
                    "SERVER #{} - Found saved info: {}",
                    received.server_num, value
                );

                let external_request = ExternalConnectionRequest {
                    request_type: ExternalConnectionRequestType::SearchReturn,
                    value: *value,
                    value_str: String::from(""),
                    return_topic: received.request_info.return_topic,
                };
                external_communicator_tx.send(external_request).unwrap();
            } else {
                println!(
                    "Unable to find saved info for key {}",
                    received.request_info.key
                );
            }
        } else if let ValueTableRequestType::DumpContents = received.request_type {
            let content_dump = ContentDumpInfo {
                payload: node_values.clone(),
            };

            let serialized_payload = serde_json::to_string(&content_dump).unwrap();
            println!("Dumping current contents: {:?}", serialized_payload);

            let external_request = ExternalConnectionRequest {
                request_type: ExternalConnectionRequestType::DumpContents,
                value: -1,
                value_str: serialized_payload,
                return_topic: format!("RESTART_SERVER_{}", received.server_num),
            };
            external_communicator_tx.send(external_request).unwrap(); // TODO: Exception handle?
        }
    }
}
