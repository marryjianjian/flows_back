mod consumer;
mod crud;
mod model;
mod read_logs;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
pub use model::{AccessInfo, AccessStatistics};
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = redis::Client::open("redis://127.0.0.1/").expect("client");
    consumer::read_records(&client).expect("simple read");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{} db_path", args[0]);
        process::exit(1);
    }
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
