use std::{
    collections::HashMap,
    env,
    process::{self, Child},
    thread,
    time::Duration,
};

use crate::{
    heartbeat_handler::{
        create_heartbeat_handler, start_server, HeartbeatRequest, HeartbeatRequestType,
    },
    mqtt_utils::DFLT_MONITOR_TOPIC,
};

mod heartbeat_handler;

mod mqtt_utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(
        args.len(),
        2,
        "The monitor must receive 1 arguments: number of servers to start"
    );

    let server_count: i32 = args[1].parse().unwrap();
    let mut servers: HashMap<i32, Child> = HashMap::new();

    // Create servers
    for i in 0..server_count {
        servers.insert(
            i,
            start_server(&i, &server_count, heartbeat_handler::ServerStartType::Boot),
        );
        println!("Created server {}", i);
    }

    // Add SigInt trap
    ctrlc::set_handler(move || {
        for i in 0..server_count {
            let child: &mut Child = servers.get_mut(&i).unwrap();
            child.kill().unwrap();
            println!("Killed server {}", i);
        }
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    // Listen monitor topic
    let mut client = mqtt_utils::get_client(&String::from("REQUESTS"));
    let topics_to_subscribe = [DFLT_MONITOR_TOPIC];
    let qos_list = [2];
    let incoming_messages =
        mqtt_utils::get_incoming_messages_iterator(&mut client, &topics_to_subscribe, &qos_list);

    // Create heartbeat handler
    let heartbeat_handler_tx = create_heartbeat_handler();
    let _heartbeat_handler_tx = heartbeat_handler_tx.clone();

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(15000));
        println!("MONITOR - Searching for dead nodes");

        let heartbeat_request = HeartbeatRequest {
            heartbeat_info: None,
            request_type: HeartbeatRequestType::SearchDeadNodes,
            server_count: server_count,
        };
        _heartbeat_handler_tx.send(heartbeat_request).unwrap();
    });

    for message in incoming_messages.iter() {
        if let Some(msg) = message {
            let payload = msg.payload_str();
            println!("MONITOR - Heartbeat: {}", payload);
            let heartbeat_info: heartbeat_handler::HeartbeatInfo =
                serde_json::from_str(&payload).unwrap();

            let heartbeat_request = HeartbeatRequest {
                heartbeat_info: Some(heartbeat_info),
                request_type: HeartbeatRequestType::Update,
                server_count: server_count,
            };

            heartbeat_handler_tx.send(heartbeat_request).unwrap();
        } else if !client.is_connected() {
            if mqtt_utils::try_reconnect(&client) {
                println!("MONITOR - Resubscribe topics...");
                if let Err(e) = client.subscribe_many(&topics_to_subscribe, &qos_list) {
                    println!("MONITOR - Error subscribes topics: {:?}", e);
                    process::exit(1);
                }
            } else {
                break;
            }
        }
    }
}
