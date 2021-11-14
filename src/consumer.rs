use crate::read_logs;
use redis;
use redis::Commands;

const TKEY: &str = "queue_test";

pub fn read_records(client: &redis::Client) -> redis::RedisResult<()> {
    let mut con = client.get_connection().expect("conn");

    loop {
        if let Some(r) = con.rpop::<String, Option<String>>(TKEY.to_string(), None)? {
            println!("rpop from {} : {}", TKEY, r);
            if let Some(access_info) = read_logs::read_access_info_fron_line(&r) {
                println!("{:?}", access_info);
            }
        } else {
            println!("return nil");
            break;
        }
    }
    Ok(())
}
