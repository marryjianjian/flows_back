mod consumer;
mod crud;
mod model;
mod read_logs;

use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use crud::update_database;
use env_logger;
use model::{AccessStatistics, DayStatistics};
use r2d2;
use r2d2_sqlite::SqliteConnectionManager;
use redis;
use rusqlite;
use std::collections::HashMap;
use std::sync::Arc;

static DEFAULT_REDIS_ADDRESS: &'static str = "redis://127.0.0.1/";
static DEFAULT_SQLITE_PATH: &'static str = "src/db/access.db";

#[derive(Debug, Clone)]
struct ConfigContext {
    redis_client: redis::Client,
    pool: crud::Pool,
}

#[get("/top10domains")]
async fn top10domains(conf_ctx: web::Data<Arc<ConfigContext>>) -> impl Responder {
    let res: Vec<AccessStatistics>;
    match crud::get_top_10_domain_statistics(&conf_ctx.pool, 10).await {
        Ok(rv) => res = rv,
        Err(err) => {
            println!("{}", err);
            res = vec![]
        }
    }
    HttpResponse::Ok().json(res)
}

#[get("/last7daystatistics")]
async fn get_last_7_days_statistics(conf_ctx: web::Data<Arc<ConfigContext>>) -> impl Responder {
    let res: Vec<DayStatistics>;
    match crud::get_last_7_days_statistics(&conf_ctx.pool).await {
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

async fn consume_redis_and_update_db(conf_ctx: &ConfigContext) -> Result<usize, rusqlite::Error> {
    let records =
        consumer::read_records(&conf_ctx.redis_client).expect("read records from redis failed");
    let updated_records_len = update_database(&conf_ctx.pool, &records).await?;

    Ok(updated_records_len)
}

use tokio::time::{self, Duration};
async fn timerf(conf_ctx: ConfigContext) {
    let mut interval = time::interval(Duration::from_secs(10));
    let (mut success_times, mut success_records_len) = (0, 0);
    loop {
        interval.tick().await;
        match consume_redis_and_update_db(&conf_ctx).await {
            Ok(updated_records_len) => {
                success_times += 1;
                success_records_len += updated_records_len;
                if success_times % 8640 == 1 {
                    println!(
                        "successfully update {} db records from redis",
                        success_records_len
                    );
                    success_records_len = 0;
                }
            }
            Err(err) => println!("update db from redis error : \"{}\"", err),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings.toml"))
        .unwrap()
        .merge(config::Environment::with_prefix("APP"))
        .unwrap();

    let (redis_address, sqlite_path): (String, String);
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
    let manager = SqliteConnectionManager::file(&sqlite_path);
    let pool = r2d2::Pool::new(manager).unwrap();

    let client = redis::Client::open(redis_address).expect("connect redis failed");
    let conf_ctx = ConfigContext {
        redis_client: client,
        pool: pool,
    };

    tokio::spawn(timerf(conf_ctx.clone()));

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(Arc::new(conf_ctx.clone())).clone())
            .service(top10domains)
            .service(get_last_7_days_statistics)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
