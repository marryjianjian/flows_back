use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct HostStatistics {
    domain: String,
    count: u64,
    something: String
}

#[get("/json")]
async fn json() -> impl Responder {
    let a = HostStatistics {
        domain: "a".to_string(),
        count: 123,
        something: "asdasdsad".to_string(),
    };
    let mut res = Vec::new();
    res.push(a);
    HttpResponse::Ok().json(res)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(json)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

