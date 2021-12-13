use std::collections::HashMap;
use std::process::Child;
use std::process::Command;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use mqtt::Client;
use serde::Deserialize;

extern crate paho_mqtt as mqtt;

use crate::mqtt_utils;
use crate::mqtt_utils::RequestInfo;
use crate::mqtt_utils::DFLT_REQ_TOPIC;

const MAX_HEARTBEAT_INTERVAL: i32 = 15; // Seconds

#[derive(Deserialize, Debug, Clone)]
pub struct HeartbeatInfo {
    pub server_number: i32,
    pub timestamp: String,
}

pub struct HeartbeatRequest {
    pub heartbeat_info: Option<HeartbeatInfo>,
    pub request_type: HeartbeatRequestType,
    pub server_count: i32,
}

pub enum HeartbeatRequestType {
    Update,
    SearchDeadNodes,
}

pub enum ServerStartType {
    Boot,    // Empty memory start
    Restart, // Server was dead, restarted with incoming state
}

pub fn create_heartbeat_handler() -> Sender<HeartbeatRequest> {
    let (tx, rx) = channel();

    std::thread::spawn(move || {
        _run_main_loop(rx);
    });

    return tx.clone();
}

fn _run_main_loop(rx: Receiver<HeartbeatRequest>) {
    let mut last_heartbeats: HashMap<i32, i32> = HashMap::new();
    let mut dead_nodes: HashMap<i32, i32> = HashMap::new();
    let heartbeat_monitor_client = mqtt_utils::get_client(&String::from("HEARTBEAT"));
    mqtt_utils::connect_client(&heartbeat_monitor_client);

    for received in rx {
        if let HeartbeatRequestType::Update = received.request_type {
            let heartbeat_info = received.heartbeat_info.unwrap();
            let server_number = heartbeat_info.server_number;

            // Verify if node was previously dead
            if dead_nodes.contains_key(&server_number) {
                // This is the restarted server's first heartbeat
                // Send NOVOSERV message
                println!(
                    "First heartbeat from freshly restarted Server #{}",
                    server_number
                );

                dead_nodes.remove(&server_number);
            }

            // Update server's last heartbeat timestamp
            last_heartbeats.insert(
                heartbeat_info.server_number,
                heartbeat_info.timestamp.parse().unwrap(),
            );
        }

        if let HeartbeatRequestType::SearchDeadNodes = received.request_type {
            let now: i32 = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i32;

            for (node_id, last_heartbeat) in &last_heartbeats {
                if now - last_heartbeat > MAX_HEARTBEAT_INTERVAL // Exceeds maximum heartbeat tolerance
                    && dead_nodes.get(node_id).unwrap_or_else(|| &0) != &-1_i32
                // Isnt already in dead nodes list
                {
                    println!("Server {} is dead! Restarting it", node_id);
                    send_heartbeat_message(
                        &heartbeat_monitor_client,
                        &String::from("FALHASERV"),
                        &node_id, // Send id of failed server
                        last_heartbeat,
                    );
                    dead_nodes.insert(*node_id, -1);
                    thread::sleep(Duration::from_millis(10000)); // Simulate restart
                    start_server(&node_id, &received.server_count, ServerStartType::Restart);
                    thread::sleep(Duration::from_millis(2000)); // Simulate startup
                    send_heartbeat_message(
                        &heartbeat_monitor_client,
                        &String::from("NOVOSERV"),
                        &node_id, // Send id of new server
                        &-1,
                    );
                }
            }
        }
    }
}

pub fn start_server(server_num: &i32, server_count: &i32, start_type: ServerStartType) -> Child {
    let start_type_string;

    match start_type {
        ServerStartType::Boot => start_type_string = "BOOT",
        ServerStartType::Restart => start_type_string = "RESTART",
    }
    Command::new("./bin/run_server.sh")
        .arg(server_num.to_string())
        .arg(server_count.to_string())
        .arg(start_type_string)
        .spawn()
        .expect("Error creating server")
}

pub fn send_heartbeat_message(
    heartbeat_monitor_client: &Client,
    req_type: &String,
    value: &i32,
    last_seen: &i32,
) {
    let request_info = RequestInfo {
        req_type: req_type.clone(),
        key: String::from(""),
        value: *value,
        return_topic: String::from(""),
        last_seen: *last_seen,
    };
    let msg = mqtt::Message::new(
        DFLT_REQ_TOPIC,
        serde_json::to_string(&request_info).unwrap(),
        2,
    );
    let res = heartbeat_monitor_client.publish(msg);

    if let Err(e) = res {
        println!("MONITOR - Error sending heartbeat message: {:?}", e);
    }
}
