use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct RequestInfo {
    pub conn_type: String, // INSERT OR SEARCH
    pub key: String,
    pub value: i32,
    pub return_topic: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct HeartbeatInfo {
    pub server_number: i32,
    pub timestamp: String,
}

pub fn is_this_node_responsability(
    request_info: &RequestInfo,
    node_responsabilities: &Vec<i32>,
    server_count: &i32,
) -> bool {
    let mut total_char_sum: i32 = 0;
    for c in request_info.key.chars() {
        total_char_sum += c as i32;
    }

    println!("{}", total_char_sum);
    for responsability in node_responsabilities {
        if total_char_sum % server_count == (*responsability) {
            // println!(
            //     "Is this node's responsability to answer search on key {}, return topic {}",
            //     request_info.key, request_info.return_topic
            // );
            return true;
        }
    }

    return false;
}
