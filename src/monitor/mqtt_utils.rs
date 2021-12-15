use std::{process, sync::mpsc::Receiver, thread, time::Duration};

use mqtt::{Client, Message};
use serde::Serialize;

extern crate paho_mqtt as mqtt;

#[derive(Serialize)]
pub struct RequestInfo {
    pub req_type: String, // INSERT / SEARCH / FALHASERV / NOVOSERV / DIE
    pub key: String,
    pub value: i32,
    pub return_topic: String,
    pub last_seen: i32,
}

const DFLT_BROKER: &str = "tcp://localhost:1883";
pub const DFLT_REQ_TOPIC: &str = "inf1406-reqs";
pub const DFLT_MONITOR_TOPIC: &str = "inf1406-mon";

pub fn try_reconnect(client: &mqtt::Client) -> bool {
    println!("MONITOR - Connection lost.");
    for _ in 0..3 {
        println!("MONITOR - Trying to reconect...");
        thread::sleep(Duration::from_millis(5000));
        if client.reconnect().is_ok() {
            println!("MONITOR - Successfully reconnected");
            return true;
        }
    }
    println!("MONITOR - Unable to reconnect after several attempts.");
    false
}

pub fn get_client(client_suffix: &String) -> Client {
    let host = DFLT_BROKER.to_string();

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(format!("MONITOR_{}", client_suffix))
        .finalize();

    let client = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("MONITOR - Error creating the client: {:?}", err);
        process::exit(1);
    });

    return client;
}
pub fn connect_client(client: &Client) {
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(200))
        .clean_session(false)
        .finalize();

    if let Err(e) = client.connect(conn_opts) {
        println!("MONITOR - Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }
}
pub fn get_incoming_messages_iterator(
    client: &mut Client,
    topics_to_subscribe: &[&str],
    qos: &[i32],
) -> Receiver<Option<Message>> {
    let rx = client.start_consuming();

    connect_client(&client);
    if let Err(e) = client.subscribe_many(topics_to_subscribe, qos) {
        println!("MONITOR - Error subscribes topics: {:?}", e);
        process::exit(1);
    }

    return rx;
}
