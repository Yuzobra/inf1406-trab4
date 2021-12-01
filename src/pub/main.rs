use std::{env, process, time::Duration};

extern crate paho_mqtt as mqtt;

const DFLT_BROKER: &str = "tcp://localhost:1883";
const DFLT_CLIENT: &str = "PUB_CLIENT";
const DFLT_TOPICS: &[&str] = &["inf1406-reqs", "inf1406-mon"];
const QOS: i32 = 2;

fn main() {
    let host = env::args()
        .nth(1)
        .unwrap_or_else(|| DFLT_BROKER.to_string());

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(DFLT_CLIENT.to_string())
        .finalize();

    let cli = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    if let Err(e) = cli.connect(conn_opts) {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }

    for num in 0..5 {
        let content = "Test message".to_string() + &num.to_string();
        let msg = mqtt::Message::new(DFLT_TOPICS[0], content.clone(), QOS);
        let tok = cli.publish(msg);

        if let Err(e) = tok {
            println!("Error sending message: {:?}", e);
            break;
        }
    }

    let tok = cli.disconnect(None);
    println!("Disconnect from the broker");
    tok.unwrap();
}
