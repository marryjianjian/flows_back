mod model;
mod read_logs;
use rusqlite::{params, Connection, Result};
use std::{env, process, time};

fn update_database(conn: &mut Connection, access_infos: &Vec<model::AccessInfo>) -> Result<()> {
    let tx = conn.transaction()?;
    // stmt need Droped first
    {
        let mut stmt = tx.prepare(
            "INSERT INTO access_info(time, src_port, src_ip, dst_port, dst_domain, state, protocol)
                VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;
        for access_info in access_infos {
            stmt.execute(params![
                access_info.time,
                access_info.src_port,
                access_info.src_ip,
                access_info.dst_port,
                access_info.dst_domain,
                access_info.state,
                access_info.protocol
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("{} db_path log_path", &args[0]);
        process::exit(1);
    }

    let now = time::Instant::now();
    let mut conn = Connection::open(&args[1]).expect("open database error");
    let access_infos =
        read_logs::read_access_info_from_file(&args[2]).expect("read log files failed");
    update_database(&mut conn, &access_infos).expect("update database error");
    println!(
        "update database success in {} us",
        now.elapsed().as_micros()
    );
}
