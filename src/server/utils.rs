use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct RequestInfo {
    pub req_type: String, // INSERT / SEARCH / FALHASERV / NOVOSERV / DIE
    pub key: String,
    pub value: i32,
    pub return_topic: String,
    pub last_seen: i32,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct HeartbeatInfo {
    pub server_number: i32,
    pub timestamp: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ContentDumpInfo {
    pub payload: HashMap<String, i32>,
}

pub fn is_this_node_search_responsability(
    request_info: &RequestInfo,
    node_responsabilities: &Vec<i32>,
    server_count: &i32,
) -> bool {
    let mut total_char_sum: i32 = 0;
    for c in request_info.key.chars() {
        total_char_sum += c as i32;
    }

    for responsability in node_responsabilities {
        if total_char_sum % server_count == (*responsability) {
            return true;
        }
    }

    return false;
}

pub fn should_become_substitute(
    request_info: &RequestInfo,
    node_responsabilities: &Vec<i32>,
    server_count: &i32,
    server_num: &i32,
) -> bool {
    let failed_server_id = request_info.value;

    if failed_server_id != *server_num {
        for responsability in node_responsabilities {
            if _should_become_subtitute(responsability, &failed_server_id, server_count) {
                return true;
            }
        }
    }

    return false;
}

fn _should_become_subtitute(
    responsability: &i32,
    failed_server_id: &i32,
    server_count: &i32,
) -> bool {
    if (failed_server_id + 1) % server_count == *responsability {
        println!(
            "Assuming responsability of failed server {}",
            failed_server_id
        );
        return true;
    }
    return false;
}
