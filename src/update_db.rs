mod crud;
mod model;
mod read_logs;
use r2d2;
use r2d2_sqlite::SqliteConnectionManager;
use std::{env, process, time};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("{} db_path log_path", &args[0]);
        process::exit(1);
    }

    let now = time::Instant::now();
    let manager = SqliteConnectionManager::file(&args[1]);
    let pool = r2d2::Pool::new(manager).unwrap();
    let access_infos =
        read_logs::read_access_info_from_file(&args[2]).expect("read log files failed");

    crud::update_database(&pool, &access_infos).await.expect("update database error");
    println!(
        "update database success in {} us",
        now.elapsed().as_micros()
    );
}
