mod consumer;
mod crud;
mod model;
mod read_logs;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use model::{AccessStatistics};
use crud::{update_database};
use redis;
use rusqlite::Connection;
use std::env;
use std::process;
use std::sync::Arc;

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

// TODO : config for redis address and db path
fn consume_redis_and_update_db(db_path : &str) -> Result<usize, rusqlite::Error> {
    let client = redis::Client::open("redis://127.0.0.1/").expect("client");
    let records = consumer::read_records(&client).expect("simple read");

    let mut conn = Connection::open(db_path).expect("open database error");

    let updated_records_len = update_database(&mut conn, &records)?;

    Ok(updated_records_len)
}

use tokio::time::{self, Duration};
async fn timerf(path : String) {
    let mut interval = time::interval(Duration::from_secs(10));
    let (mut success_times, mut success_records_len) = (0, 0);
    loop {
        interval.tick().await;
        match consume_redis_and_update_db(&path) {
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

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{} db_path", args[0]);
        process::exit(1);
    }

    tokio::spawn(timerf(args[1].clone()));

    let db_path = web::Data::new(Arc::new(args[1].clone()));

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
