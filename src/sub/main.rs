use std::{env, process, thread, time::Duration};

extern crate paho_mqtt as mqtt;

const DFLT_BROKER: &str = "tcp://localhost:1883";
const DFLT_TOPICS: &[&str] = &["RESTART_SERVER_3", "inf1406-reqs"];
const DFLT_QOS: &[i32] = &[2, 2];

fn try_reconnect(client: &mqtt::Client) -> bool {
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

fn subscribe_topics(client: &mqtt::Client) {
    if let Err(e) = client.subscribe_many(DFLT_TOPICS, DFLT_QOS) {
        println!("Error subscribes topics: {:?}", e);
        process::exit(1);
    }
}

fn main() {
    let host = DFLT_BROKER.to_string();

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(format!(
            "SUB_CLIENT_{}",
            env::args().nth(1).unwrap_or_else(|| "0".to_string())
        ))
        .finalize();

    let mut client = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let rx = client.start_consuming();

    let lwt = mqtt::MessageBuilder::new()
        .topic("test")
        .payload("Consumer lost connection")
        .finalize();
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(false)
        .will_message(lwt)
        .finalize();

    if let Err(e) = client.connect(conn_opts) {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }

    subscribe_topics(&client);

    println!("Processing requests...");
    for msg in rx.iter() {
        if let Some(msg) = msg {
            println!("{}", msg);
        } else if !client.is_connected() {
            if try_reconnect(&client) {
                println!("Resubscribe topics...");
                subscribe_topics(&client);
            } else {
                break;
            }
        }
    }

    // If still connected, then disconnect now.
    if client.is_connected() {
        println!("Disconnecting");
        client.unsubscribe_many(DFLT_TOPICS).unwrap();
        client.disconnect(None).unwrap();
    }
    println!("Exiting");
}
