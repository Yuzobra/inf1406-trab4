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

use crate::mqtt_utils::connect_client;
use crate::mqtt_utils::get_client;
use crate::mqtt_utils::DFLT_MONITOR_TOPIC;
use crate::utils::HeartbeatInfo;
extern crate paho_mqtt as mqtt;

// use crate::utils::RequestInfo;

pub struct ExternalConnectionRequest {
    pub request_type: ExternalConnectionRequestType,
    pub value: i32,
    pub return_topic: String,
}

pub enum ExternalConnectionRequestType {
    SearchReturn,
    NodeSearchFollow,
}

pub fn create_external_communicator_thread(server_num: &i32) -> Sender<ExternalConnectionRequest> {
    let (tx, rx) = channel();

    let external_comm_client = get_client(server_num, Some(String::from("SERVER_EXTERNAL_COMM")));
    connect_client(&external_comm_client);
    std::thread::spawn(move || {
        _run_main_loop(rx, &external_comm_client);
    });

    let heartbeat_client = get_client(server_num, Some(String::from("SERVER_HEARBEAT")));
    connect_client(&heartbeat_client);
    let _server_num = (*server_num).clone();
    std::thread::spawn(move || loop {
        _run_heartbeat_send(&heartbeat_client, &_server_num);
        thread::sleep(Duration::from_millis(4000));
    });

    return tx.clone();
}

fn _run_main_loop(rx: Receiver<ExternalConnectionRequest>, client: &Client) {
    for received in rx {
        println!("Received external connection request");
        if let ExternalConnectionRequestType::SearchReturn = received.request_type {
            let msg = mqtt::Message::new(received.return_topic, received.value.to_string(), 2);
            let res = client.publish(msg);

            if let Err(e) = res {
                println!("Error sending message to waiting client: {:?}", e);
                process::exit(1);
            }
        } else if let ExternalConnectionRequestType::NodeSearchFollow = received.request_type {
            // if let Ok(mut stream) = TcpStream::connect(&received.conn_ip) {
            //     let serialized_info = serde_json::to_string(&received.conn_info).unwrap();
            //     let res = stream.write(serialized_info.as_bytes());
            //     match res {
            //         Ok(_) => println!("Succesfully forwarded info to {}", received.conn_ip),
            //         Err(_) => {
            //             println!("Error");
            //             process::exit(0x0)
            //         }
            //     }
            // } else {
            //     println!("Couldn't connect to server in {}", received.conn_ip);
            // }
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
