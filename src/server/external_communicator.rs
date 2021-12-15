use std::process;
// use std::io::prelude::*;
// use std::net::TcpStream;
// use std::process;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use paho_mqtt::Client;
use serde::Serialize;

use crate::mqtt_utils::connect_client;
use crate::mqtt_utils::get_client;
use crate::mqtt_utils::DFLT_MONITOR_TOPIC;
use crate::utils::HeartbeatInfo;
extern crate paho_mqtt as mqtt;

// use crate::utils::RequestInfo;

pub struct ExternalConnectionRequest {
    pub request_type: ExternalConnectionRequestType,
    pub value: i32,
    pub value_str: String,
    pub return_topic: String,
}

pub enum ExternalConnectionRequestType {
    SearchReturn,
    DumpContents,
}

#[derive(Serialize)]
pub struct SearchReturnInfo {
    pub value: i32,
    pub return_server: i32, // Server ID of returning server
}

pub fn create_external_communicator_thread(server_num: &i32) -> Sender<ExternalConnectionRequest> {
    let (tx, rx) = channel();

    let external_comm_client = get_client(server_num, Some(String::from("SERVER_EXTERNAL_COMM")));
    connect_client(&external_comm_client);
    let mut _server_num = (*server_num).clone();
    std::thread::spawn(move || {
        _run_main_loop(rx, &external_comm_client, &_server_num);
    });

    let heartbeat_client = get_client(server_num, Some(String::from("SERVER_HEARTBEAT")));
    connect_client(&heartbeat_client);
    _server_num = (*server_num).clone();
    std::thread::spawn(move || loop {
        _run_heartbeat_send(&heartbeat_client, &_server_num);
        thread::sleep(Duration::from_millis(4000));
    });

    return tx.clone();
}

fn _run_main_loop(rx: Receiver<ExternalConnectionRequest>, client: &Client, server_num: &i32) {
    for received in rx {
        println!(
            "SERVER #{} - Received external connection request",
            server_num
        );
        if let ExternalConnectionRequestType::SearchReturn = received.request_type {
            let search_return_info = SearchReturnInfo {
                value: received.value,
                return_server: server_num.clone(),
            };
            let msg = mqtt::Message::new(
                received.return_topic,
                serde_json::to_string(&search_return_info).unwrap(),
                2,
            );
            let res = client.publish(msg);

            if let Err(e) = res {
                println!(
                    "SERVER #{} - sending message to waiting client: {:?}",
                    server_num, e
                );
                process::exit(1);
            }
        } else if let ExternalConnectionRequestType::DumpContents = received.request_type {
            println!(
                "SERVER #{} - Dumping to topic {}",
                server_num, received.return_topic
            );
            let msg = mqtt::Message::new(received.return_topic, received.value_str, 2);
            let res = client.publish(msg);
            if let Err(e) = res {
                println!("Error dumping contents: {:?}", e);
                process::exit(1);
            }
        }
    }
}

fn _run_heartbeat_send(client: &Client, server_num: &i32) {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let heartbeat_info = serde_json::to_string(&HeartbeatInfo {
        server_number: *server_num,
        timestamp: timestamp.as_secs().to_string(),
    })
    .unwrap();

    let msg = mqtt::Message::new(DFLT_MONITOR_TOPIC, heartbeat_info, 2);
    let res = client.publish(msg);
    if let Err(e) = res {
        println!("Error sending heartbeat: {:?}", e);
        process::exit(1);
    }
}
