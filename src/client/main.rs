use mqtt_utils::send_data;
use paho_mqtt::Client;
use serde::{Deserialize, Serialize};
use std::{env, process};

mod mqtt_utils;

#[derive(Serialize)]
pub struct RequestInfo {
    pub req_type: String, // INSERT / SEARCH / FALHASERV / NOVOSERV / DIE
    pub key: String,
    pub value: i32,
    pub return_topic: String,
    pub last_seen: i32,
}

#[derive(Deserialize)]
pub struct SearchReturnInfo {
    pub value: i32,
    pub return_server: i32, // Server ID of returning server
}

const REQ_TOPIC: &str = "inf1406-reqs";

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(
        args.len(),
        7,
        "The server must receive 6 arguments: INSERT or SEARCH, the key, the value to insert, the client number, and expected SEARCH number, expected server to respond."
    );
    let operation = args[1].clone();
    let key = args[2].clone();
    let value: i32 = args[3].parse().unwrap();
    let client_num: i32 = args[4].parse().unwrap();
    let expected_search_value: i32 = args[5].parse().unwrap();
    let expected_return_server: i32 = args[6].parse().unwrap();
    let return_topic: String = format!("CLIENT_TOPIC_{}", client_num);

    let mut client = mqtt_utils::get_client(client_num);

    println!(
        "Client #{} -- Sending {} request:\tKey {} \tValue: {}",
        client_num, operation, key, value
    );

    if operation == String::from("INSERT") {
        _insert_value(&client, key, value, return_topic);
    } else if operation == String::from("SEARCH") {
        _search_value(&client, key, &return_topic);
        let received_value = _receive_searched_value(&mut client, &return_topic);
        println!("Received return value: {}", received_value.value);

        if expected_search_value != -1 {
            if expected_search_value != received_value.value {
                println!(
                    "ERROR: Received return value and expected value mismatch:\nExpected:\t{}\nReceived:\t{}",
                    expected_search_value,
                    received_value.value
                );
                process::exit(1);
            } else {
                println!(
                    "Received and return value matching: {}",
                    received_value.value
                );
            }
        }
        if expected_return_server != -1 {
            if expected_return_server != received_value.return_server {
                println!(
                    "ERROR: The server that responded and the expected mismatch:\nExpected:\t{}\nReceived:\t{}",
                    expected_return_server,
                    received_value.return_server
                );
                process::exit(1);
            } else {
                println!(
                    "Return server matches expected: {}",
                    received_value.return_server
                );
            }
        }
    } else if operation == String::from("DIE") {
        let req_type = String::from("DIE");
        let request_info = RequestInfo {
            req_type: req_type,
            key: String::from(""),
            value: value,
            return_topic: String::from(""),
            last_seen: -1,
        };
        let serialized_info = serde_json::to_string(&request_info).unwrap();
        send_data(&client, REQ_TOPIC, serialized_info);
    }
}

fn _insert_value(client: &Client, key: String, value: i32, return_topic: String) {
    let req_type = String::from("INSERT");
    let request_info = RequestInfo {
        req_type: req_type,
        key: key,
        value: value,
        return_topic: return_topic,
        last_seen: -1,
    };
    let serialized_info = serde_json::to_string(&request_info).unwrap();
    send_data(&client, REQ_TOPIC, serialized_info);
}

fn _search_value(client: &Client, key: String, return_topic: &String) {
    let req_type = String::from("SEARCH");

    let request_info = RequestInfo {
        req_type: req_type,
        key: key,
        value: -1,
        return_topic: return_topic.clone(),
        last_seen: -1,
    };
    let serialized_info = serde_json::to_string(&request_info).unwrap();
    send_data(&client, REQ_TOPIC, serialized_info);
}

fn _receive_searched_value(client: &mut Client, return_topic: &String) -> SearchReturnInfo {
    if let Err(e) = client.subscribe(return_topic.as_str(), 2) {
        println!("Error subscribes topics: {:?}", e);
    }
    let rx = client.start_consuming();
    let mut return_value = SearchReturnInfo {
        value: -1,
        return_server: -1,
    };
    println!("CLIENT - Waiting for search results");
    for message in rx.iter() {
        if let Some(msg) = message {
            println!("Received results: {}", msg.payload_str());
            return_value = serde_json::from_str(&msg.payload_str()).unwrap();
            break;
        } else if !client.is_connected() {
            println!("CLIENT - Error while waiting for search response");
        }
    }
    return return_value;
}
