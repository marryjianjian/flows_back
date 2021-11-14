use crate::read_logs;
use crate::model::{AccessInfo};
use redis;
use redis::Commands;

// TODO: config
const TKEY: &str = "queue_test";

pub fn read_records(client: &redis::Client) -> redis::RedisResult<Vec<AccessInfo>> {
    let mut con = client.get_connection().expect("conn");
    let mut res : Vec<AccessInfo> = Vec::new();

    loop {
        if let Some(r) = con.rpop::<String, Option<String>>(TKEY.to_string(), None)? {
            if let Some(access_info) = read_logs::read_access_info_fron_line(&r) {
                res.push(access_info);
            }
        } else {
            break;
        }
    }
    Ok(res)
}
