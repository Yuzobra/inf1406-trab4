use std::{process, sync::mpsc::Receiver, thread, time::Duration};

use mqtt::{Client, Message};

extern crate paho_mqtt as mqtt;

const DFLT_BROKER: &str = "tcp://localhost:1883";
pub const DFLT_REQ_TOPIC: &str = "inf1406-reqs";
pub const DFLT_MONITOR_TOPIC: &str = "inf1406-mon";
const DFLT_TOPICS: &[&str] = &[DFLT_REQ_TOPIC, DFLT_MONITOR_TOPIC];
const DFLT_QOS: &[i32] = &[2, 2];

pub fn try_reconnect(client: &mqtt::Client) -> bool {
    println!("Connection lost.");
    for _ in 0..3 {
        println!("Trying to reconect...");
        thread::sleep(Duration::from_millis(5000));
        if client.reconnect().is_ok() {
            println!("Successfully reconnected");
            return true;
        }
    }
    println!("Unable to reconnect after several attempts.");
    false
}

pub fn get_client() -> Client {
    let host = DFLT_BROKER.to_string();

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id("MONITOR")
        .finalize();

    let client = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("MONITOR - Error creating the client: {:?}", err);
        process::exit(1);
    });

    return client;
}

pub fn get_incoming_messages_iterator(
    client: &mut Client,
    topics_to_subscribe: &[&str],
    QOS: &[i32],
) -> Receiver<Option<Message>> {
    let rx = client.start_consuming();

    connect_client(&client);
    if let Err(e) = client.subscribe_many(topics_to_subscribe, QOS) {
        println!("MONITOR - Error subscribes topics: {:?}", e);
        process::exit(1);
    }

    return rx;
}

pub fn connect_client(client: &Client) {
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(false)
        .finalize();

    if let Err(e) = client.connect(conn_opts) {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }
}
