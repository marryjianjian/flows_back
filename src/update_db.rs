mod crud;
mod model;
mod read_logs;
use rusqlite::Connection;
use std::{env, process, time};

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
    crud::update_database(&mut conn, &access_infos).expect("update database error");
    println!(
        "update database success in {} us",
        now.elapsed().as_micros()
    );
}
