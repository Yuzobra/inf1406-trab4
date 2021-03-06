use std::collections::HashMap;
use std::env;
use std::process;
use std::sync::mpsc::Sender;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

mod external_communicator;
mod mqtt_utils;
mod utils;
mod values_table_handler;

use crate::mqtt_utils::DFLT_REQ_TOPIC;

use crate::external_communicator::ExternalConnectionRequest;
use crate::utils::ContentDumpInfo;
// use crate::external_communicator::ExternalConnectionRequestType;
use crate::utils::is_this_node_search_responsability;
use crate::utils::RequestInfo;
// use crate::utils::SearchResult;
use crate::values_table_handler::ValueTableRequest;
use crate::values_table_handler::ValueTableRequestType;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(
        args.len(),
        4,
        "The server must receive 3 arguments: its number, total server count, start type (BOOT or RESTART)"
    );

    let server_num: i32 = args[1].parse().unwrap();
    let server_count: i32 = args[2].parse().unwrap();
    let start_type: String = args[3].clone();

    // Create table with this node's responsabilities
    let mut node_responsabilities: Vec<i32> = [server_num].to_vec(); // Start only with this node

    // Create search log table and hashtable handler thread
    let mut search_log: HashMap<i32, RequestInfo> = HashMap::new();
    let values_table_tx: Sender<ValueTableRequest>;
    let mut node_values: HashMap<String, i32> = HashMap::new();
    if start_type == "RESTART" {
        println!(
            "SERVER #{} - Restart started, waiting for existing state on topic RESTART_SERVER_{}",
            server_num, server_num
        );
        let message_raw: String = mqtt_utils::get_last_message_from_topic(
            format!("RESTART_SERVER_{}", server_num),
            &server_num,
        );
        println!("SERVER #{} - Received state {}", server_num, message_raw);
        let parsed_message: ContentDumpInfo = serde_json::from_str(&message_raw).unwrap();
        node_values = parsed_message.payload.clone();
    }

    // Create external communicator thread
    let external_communicator_tx: Sender<ExternalConnectionRequest> =
        external_communicator::create_external_communicator_thread(&server_num);

    values_table_tx = values_table_handler::create_valuetable_handler(
        node_values,
        external_communicator_tx.clone(),
    );

    // Listen requests topic
    let mut client = mqtt_utils::get_client(&server_num, None);
    let topics_to_subscribe = [DFLT_REQ_TOPIC];
    let qos_list = [2];
    let incoming_messages =
        mqtt_utils::get_incoming_messages_iterator(&mut client, &topics_to_subscribe, &qos_list);

    println!("Server #{} ready", server_num);
    for msg in incoming_messages.iter() {
        if let Some(msg) = msg {
            let topic = msg.topic();
            let payload = msg.payload_str();
            let request_info: RequestInfo = serde_json::from_str(&payload).unwrap();

            let _values_table_tx = values_table_tx.clone();
            let _external_communicator_tx = external_communicator_tx.clone();

            if topic == DFLT_REQ_TOPIC {
                // Incoming request
                handle_request(
                    request_info,
                    _values_table_tx,
                    // _external_communicator_tx,
                    &mut node_responsabilities,
                    &server_count,
                    &mut search_log,
                    &server_num,
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
    node_responsabilities: &mut Vec<i32>,
    server_count: &i32,
    search_log: &mut HashMap<i32, RequestInfo>,
    server_num: &i32,
) {
    if request_info.req_type == "INSERT" {
        println!(
            "Incoming insert request:\tKey {}\tValue:{}",
            request_info.key, request_info.value
        );
        let value_table_request_type = ValueTableRequestType::Insert;
        let value_table_request = ValueTableRequest {
            request_type: value_table_request_type,
            request_info: request_info,
            server_num: *server_num,
        };
        values_table_tx.send(value_table_request).unwrap();
    } else if request_info.req_type == "SEARCH" {
        if utils::is_this_node_search_responsability(
            &request_info,
            node_responsabilities,
            server_count,
        ) {
            println!(
                "SERVER #{} - Responding search to key {}",
                server_num, request_info.key
            );
            let value_table_request_type = ValueTableRequestType::Search;
            let value_table_request = ValueTableRequest {
                request_type: value_table_request_type,
                request_info: request_info.clone(),
                server_num: *server_num,
            };
            values_table_tx.send(value_table_request).unwrap();
        }

        // Add entry to search logs
        search_log.insert(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i32,
            request_info.clone(),
        );
    } else if request_info.req_type == "FALHASERV" {
        println!("SERVER #{} - Recebido requisi????o de FALHASERV", server_num);
        if utils::should_become_substitute(
            &request_info,
            node_responsabilities,
            server_count,
            server_num,
        ) {
            println!(
                "SERVER #{} - Virou respons??vel por {}",
                server_num, request_info.value
            );
            // Inserir responsabilidade nas responsabilities
            node_responsabilities.insert(0, request_info.value);

            // Verificar se tem alguma mensagem n??o enviada desde o ultimo heartbeat
            for (timestamp, saved_request_info) in search_log {
                if timestamp > &request_info.last_seen
                    && is_this_node_search_responsability(
                        saved_request_info,
                        node_responsabilities,
                        server_count,
                    )
                {
                    println!(
                        "Server #{} - Found non answered search on key {}",
                        server_num, saved_request_info.key
                    );
                    let value_table_request_type = ValueTableRequestType::Search;
                    let value_table_request = ValueTableRequest {
                        request_type: value_table_request_type,
                        request_info: saved_request_info.clone(),
                        server_num: *server_num,
                    };
                    values_table_tx.send(value_table_request).unwrap();
                }
            }
        }
    } else if request_info.req_type == "NOVOSERV" {
        println!("SERVER #{} - Received NOVOSERV request", server_num);
        let novoserv_num = request_info.value.clone();
        if node_responsabilities.contains(&novoserv_num) && novoserv_num != *server_num {
            let new_server_num = novoserv_num;
            println!(
                "SERVER #{} - Is responsible for NOVOSERV {}",
                server_num, new_server_num
            );

            // Send current state to new server
            let value_table_request = ValueTableRequest {
                request_info: request_info,
                request_type: ValueTableRequestType::DumpContents,
                server_num: new_server_num,
            };
            values_table_tx.send(value_table_request).unwrap();

            // Remove NOVOSERV from responsabilities
            println!(
                "SERVER #{} - Removing {} from responsabilities. Responsabilities before: {:?}",
                server_num, new_server_num, node_responsabilities
            );
            let index = node_responsabilities
                .iter()
                .position(|x| *x == new_server_num)
                .unwrap();
            node_responsabilities.remove(index);
            println!(
                "SERVER #{} - Responsabilities after: {:?}",
                server_num, node_responsabilities
            );
        }
    } else if request_info.req_type == "DIE"
    /* DEBUG FEATURE */
    {
        if request_info.value == *server_num {
            println!("SERVER #{} - Received DIE event, exiting...", server_num);
            process::exit(0);
        }
    } else {
        println!(
            "Invalid {} req_type received, only \"INSERT\" and \"SEARCH\" are valid",
            request_info.req_type
        );
    }
}
