use mqtt_utils::send_data;
use paho_mqtt::Client;
use serde::Serialize;
use std::env;

mod mqtt_utils;

#[derive(Serialize)]
pub struct RequestInfo {
    pub req_type: String, // INSERT / SEARCH / FALHASERV / NOVOSERV
    pub key: String,
    pub value: i32,
    pub return_topic: String,
    pub last_seen: i32,
}
const REQ_TOPIC: &str = "inf1406-reqs";

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(
        args.len(),
        5,
        "The server must receive 4 arguments: INSERT or SEARCH, the key, the value to insert, the client number."
    );
    let operation = args[1].clone();
    let key = args[2].clone();
    let value: i32 = args[3].parse().unwrap();
    let client_num: i32 = args[4].parse().unwrap();
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
        println!(
            "Received return value: {}",
            _receive_searched_value(&mut client, &return_topic)
        );
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

fn _receive_searched_value(client: &mut Client, return_topic: &String) -> i32 {
    if let Err(e) = client.subscribe(return_topic.as_str(), 2) {
        println!("Error subscribes topics: {:?}", e);
    }
    let rx = client.start_consuming();
    let mut return_value: i32 = -1;
    println!("CLIENT - Waiting for search results");
    for message in rx.iter() {
        if let Some(msg) = message {
            println!("{}", msg);
            return_value = msg.payload_str().parse().unwrap();
            break;
        } else if !client.is_connected() {
            println!("CLIENT - Error while waiting for search response");
        }
    }
    return return_value;
}
