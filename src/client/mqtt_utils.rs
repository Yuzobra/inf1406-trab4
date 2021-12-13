use std::{process, time::Duration};

use mqtt::Client;

extern crate paho_mqtt as mqtt;

const DFLT_BROKER: &str = "tcp://localhost:1883";
// const DFLT_CLIENT: &str = "PUB_CLIENT";
// const DFLT_TOPICS: &[&str] = &["inf1406-reqs", "inf1406-mon"];
const QOS: i32 = 2;

pub fn get_client(client_num: i32) -> Client {
    let host = DFLT_BROKER.to_string();

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(format!("CLIENT_{}", client_num))
        .finalize();

    let client = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    if let Err(e) = client.connect(conn_opts) {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }

    return client;
}

pub fn send_data(client: &Client, topic: &str, message: String) {
    let msg = mqtt::Message::new(topic, message, QOS);
    let res = client.publish(msg);

    if let Err(e) = res {
        println!("Error sending message: {:?}", e);
    }
}

pub fn disconnect_client(client: Client) {
    let tok = client.disconnect(None);
    println!("Disconnect from the broker");
    tok.unwrap();
}
