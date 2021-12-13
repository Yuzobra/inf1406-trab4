use std::collections::HashMap;
// use std::collections::HashMap;
use std::env;
use std::process;
// use std::io::Read;
// use std::io::Write;
// use std::net::TcpListener;
// use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::time::SystemTime;
// use std::thread;

mod external_communicator;
mod mqtt_utils;
mod utils;
mod values_table_handler;

use crate::mqtt_utils::DFLT_REQ_TOPIC;

use crate::external_communicator::ExternalConnectionRequest;
// use crate::external_communicator::ExternalConnectionRequestType;
use crate::utils::RequestInfo;
// use crate::utils::SearchResult;
use crate::values_table_handler::ValueTableRequest;
use crate::values_table_handler::ValueTableRequestType;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(
        args.len(),
        3,
        "The server must receive 2 arguments: its number and total server count"
    );

    let server_num: i32 = args[1].parse().unwrap();
    let server_count: i32 = args[2].parse().unwrap();

    // Create search log table
    let mut search_log: HashMap<SystemTime, RequestInfo> = HashMap::new();

    // Create table with this node's responsabilities
    let node_responsabilities: Vec<i32> = [server_num].to_vec(); // Start only with this node

    // Create external communicator thread
    let external_communicator_tx: Sender<ExternalConnectionRequest> =
        external_communicator::create_external_communicator_thread(&server_num);

    // Create hashtable handler thread
    let values_table_tx: Sender<ValueTableRequest> =
        values_table_handler::create_valuetable_handler(external_communicator_tx.clone());

    // Listen requests topic
    let mut client = mqtt_utils::get_client(&server_num, None);
    let topics_to_subscribe = [DFLT_REQ_TOPIC];
    let qos_list = [2];
    let incoming_messages =
        mqtt_utils::get_incoming_messages_iterator(&mut client, &topics_to_subscribe, &qos_list);

    println!("Server #{} receiving messages", server_num);
    for msg in incoming_messages.iter() {
        if let Some(msg) = msg {
            let topic = msg.topic();
            let payload = msg.payload_str();
            println!("{}", payload);
            let request_info: RequestInfo = serde_json::from_str(&payload).unwrap();

            let _values_table_tx = values_table_tx.clone();
            let _external_communicator_tx = external_communicator_tx.clone();

            if topic == DFLT_REQ_TOPIC {
                // Incoming request
                handle_request(
                    request_info,
                    _values_table_tx,
                    // _external_communicator_tx,
                    &node_responsabilities,
                    &server_count,
                    &mut search_log,
                );
            }
        } else if !client.is_connected() {
            if mqtt_utils::try_reconnect(&client) {
                println!("Resubscribe topics...");
                if let Err(e) = client.subscribe_many(&topics_to_subscribe, &qos_list) {
                    println!("SERVER - Error subscribes topics: {:?}", e);
                    process::exit(1);
                }
            } else {
                break;
            }
        }
    }
}

fn handle_request(
    request_info: RequestInfo,
    values_table_tx: Sender<ValueTableRequest>,
    // external_communicator_tx: Sender<ExternalConnectionRequest>,
    node_responsabilities: &Vec<i32>,
    server_count: &i32,
    search_log: &mut HashMap<SystemTime, RequestInfo>,
) {
    if request_info.conn_type == "INSERT" {
        println!(
            "Incoming insert request:\tKey {}\tValue:{}",
            request_info.key, request_info.value
        );
        let value_table_request_type = ValueTableRequestType::Insert;
        let value_table_request = ValueTableRequest {
            request_type: value_table_request_type,
            request_info: request_info,
        };
        values_table_tx.send(value_table_request).unwrap();
    } else if request_info.conn_type == "SEARCH" {
        if utils::is_this_node_responsability(&request_info, node_responsabilities, server_count) {
            let value_table_request_type = ValueTableRequestType::Search;
            let value_table_request = ValueTableRequest {
                request_type: value_table_request_type,
                request_info: request_info.clone(),
            };
            values_table_tx.send(value_table_request).unwrap();
        }

        // Add entry to search logs
        search_log.insert(SystemTime::now(), request_info.clone());
    } else {
        println!(
            "Invalid {} conn_type received, only \"INSERT\" and \"SEARCH\" are valid",
            request_info.conn_type
        );
    }
}
