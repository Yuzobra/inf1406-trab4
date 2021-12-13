use std::{
    collections::HashMap,
    env,
    process::{self, Child, Command},
};

use crate::mqtt_utils::DFLT_MONITOR_TOPIC;

use crate::heartbeat_utils::HeartbeatInfo;

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

    for i in 0..server_count {
        servers.insert(
            i,
            Command::new("./bin/run_server.sh")
                .arg(i.to_string())
                .arg(server_count.to_string())
                .spawn()
                .expect("Error creating server"),
        );
        println!("Created server {}", i);
    }

    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        for i in 0..server_count {
            let child: &mut Child = servers.get_mut(&i).unwrap();
            child.kill().unwrap();
            println!("Killed server {}", i);
        }
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    // Listen monitor topic
    let mut client = mqtt_utils::get_client();
    let topics_to_subscribe = [DFLT_MONITOR_TOPIC];
    let qos_list = [2];
    let incoming_messages =
        mqtt_utils::get_incoming_messages_iterator(&mut client, &topics_to_subscribe, &qos_list);

    for message in incoming_messages.iter() {
        if let Some(msg) = message {
            let payload = msg.payload_str();
            println!("MONITOR - {}", payload);
            let heartbeat_info: HeartbeatInfo = serde_json::from_str(&payload).unwrap();
            let timestamp: i32 = heartbeat_info.timestamp.parse().unwrap();

            println!("Timestamp {:?}", timestamp);
            last_heartbeats.insert(heartbeat_info.server_number, timestamp);
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
