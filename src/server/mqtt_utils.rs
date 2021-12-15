use std::{process, sync::mpsc::Receiver, thread, time::Duration};
use uuid::Uuid;

use mqtt::{Client, Message};

extern crate paho_mqtt as mqtt;

const DFLT_BROKER: &str = "tcp://localhost:1883";
pub const DFLT_REQ_TOPIC: &str = "inf1406-reqs";
pub const DFLT_MONITOR_TOPIC: &str = "inf1406-mon";

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

pub fn get_client(server_num: &i32, base_name: Option<String>) -> Client {
    let host = DFLT_BROKER.to_string();

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(format!(
            "{}_{}_{}",
            base_name.unwrap_or(String::from("SERVER")),
            server_num,
            Uuid::new_v4()
        ))
        .finalize();

    let client = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("SERVER - Error creating the client: {:?}", err);
        process::exit(1);
    });

    return client;
}

pub fn get_incoming_messages_iterator(
    client: &mut Client,
    topics_to_subscribe: &[&str],
    qos: &[i32],
) -> Receiver<Option<Message>> {
    let rx = client.start_consuming();

    connect_client(&client);
    if let Err(e) = client.subscribe_many(topics_to_subscribe, qos) {
        println!("SERVER - Error subscribes topics: {:?}", e);
        process::exit(1);
    }

    return rx;
}

pub fn get_last_message_from_topic(topic_name: String, server_num: &i32) -> String {
    let mut client = get_client(&server_num, Some(String::from("SERVER_SINGLE_MESSAGE")));
    let topics_to_subscribe = [topic_name.as_str()];
    let qos_list = [2];
    let incoming_messages =
        get_incoming_messages_iterator(&mut client, &topics_to_subscribe, &qos_list);
    println!(
        "SERVER #{} - Getting last message from topic {}",
        server_num, topic_name
    );
    let mut return_message: String = String::from("");
    let mut iter = incoming_messages.try_iter();
    let mut message = iter.next();
    while !message.is_none() || return_message == "" {
        thread::sleep(Duration::from_millis(100));
        // for msg in incoming_messages.recv_timeout(Duration::from_secs(3)) {
        if let Some(_message) = message {
            if !_message.is_none() {
                let msg = _message.unwrap();
                return_message = msg.payload_str().parse().unwrap();
                println!(
                    "SERVER #{} - Found one message from topic {}: {}",
                    server_num, topic_name, return_message
                );
            }
        } else if !client.is_connected() {
            if try_reconnect(&client) {
                println!("Resubscribe topics...");
                if let Err(e) = client.subscribe_many(&topics_to_subscribe, &qos_list) {
                    println!("SERVER - Error subscribes topics: {:?}", e);
                    process::exit(1);
                }
            } else {
                println!("SERVER - Error connecting to server on get_last_message_from_topic");
                process::exit(1);
            }
        }
        message = iter.next();
    }
    return return_message;
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
