mod consumer;
mod crud;
mod model;
mod read_logs;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use model::{AccessStatistics};
use crud::{update_database};
use redis;
use rusqlite::Connection;
use std::sync::Arc;
use std::collections::HashMap;

static DEFAULT_REDIS_ADDRESS: &'static str = "redis://127.0.0.1/";
static DEFAULT_SQLITE_PATH: &'static str = "src/db/access.db";

#[get("/json")]
async fn json(db_path: web::Data<Arc<String>>) -> impl Responder {
    // TODO: optimize connection of database
    let conn = Connection::open(&**db_path.as_ref()).expect("open database error");

    let res: Vec<AccessStatistics>;
    match crud::get_access_statistics(&conn, 10) {
        Ok(rv) => res = rv,
        Err(err) => {
            println!("{}", err);
            res = vec![]
        }
    }
    HttpResponse::Ok().json(res)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

fn consume_redis_and_update_db(db_path: &str, redis_address: &str) -> Result<usize, rusqlite::Error> {
    let client = redis::Client::open(redis_address).expect("connect redis failed");
    let records = consumer::read_records(&client).expect("read records from redis failed");

    let mut conn = Connection::open(db_path).expect("open database error");

    let updated_records_len = update_database(&mut conn, &records)?;

    Ok(updated_records_len)
}

use tokio::time::{self, Duration};
async fn timerf(db_path: String, redis_address: String) {
    let mut interval = time::interval(Duration::from_secs(10));
    let (mut success_times, mut success_records_len) = (0, 0);
    loop {
        interval.tick().await;
        match consume_redis_and_update_db(&db_path, &redis_address) {
            Ok(updated_records_len) => {
                success_times += 1;
                success_records_len += updated_records_len;
                if success_times % 8640 == 1 {
                    println!("successfully update {} db records from redis", success_records_len);
                    success_records_len = 0;
                }
            },
            Err(err) => println!("update db from redis error : \"{}\"", err),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings.toml")).unwrap()
        .merge(config::Environment::with_prefix("APP")).unwrap();

    let (redis_address, sqlite_path) : (String, String);
    let conf = settings.try_into::<HashMap<String, String>>().unwrap();
    match conf.get("redis_address") {
        Some(res) => redis_address = res.to_string(),
        None => {
            println!("No redis address, use default : {}", DEFAULT_REDIS_ADDRESS);
            redis_address = DEFAULT_REDIS_ADDRESS.to_string();
        }
    }
    match conf.get("sqlite_path") {
        Some(res) => sqlite_path = res.to_string(),
        None => {
            println!("No sqlite db path, use default : {}", DEFAULT_SQLITE_PATH);
            sqlite_path = DEFAULT_SQLITE_PATH.to_string();
        }
    }
    println!("{}, {}", redis_address, sqlite_path);

    tokio::spawn(timerf(sqlite_path.clone(), redis_address));

    let db_path = web::Data::new(Arc::new(sqlite_path));

    HttpServer::new(move || {
        App::new()
            .app_data(db_path.clone())
            .service(json)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
