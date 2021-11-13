mod model;
mod read_logs;
use rusqlite::{params, Connection, Result};
use std::{env, process};

fn update_database(conn: &Connection, access_infos: &Vec<model::AccessInfo>) -> Result<()> {
    for access_info in access_infos {
        conn.execute(
            "INSERT INTO access_info(time, src_port, src_ip, dst_port, dst_domain, state, protocol)
                VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                access_info.time,
                access_info.src_port,
                access_info.src_ip,
                access_info.dst_port,
                access_info.dst_domain,
                access_info.state,
                access_info.protocol
            ],
        )?;
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("{} db_path log_path", &args[0]);
        process::exit(1);
    }

    let conn = Connection::open(&args[1]).expect("open database error");
    let access_infos =
        read_logs::read_access_info_from_file(&args[2]).expect("read log files failed");
    update_database(&conn, &access_infos).expect("update database error");
    println!("update database success");
}
