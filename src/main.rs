mod model;
mod read_logs;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
pub use model::{AccessInfo, AccessStatistics};
use rusqlite::{params, Connection, Result};
use std::env;
use std::process;
use std::sync::Arc;

#[get("/json")]
async fn json(db_path: web::Data<Arc<String>>) -> impl Responder {
    // TODO: optimize connection of database
    let conn = Connection::open(&**db_path.as_ref()).expect("open database error");

    let res: Vec<AccessStatistics>;
    match get_access_statistics(&conn, 10) {
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

fn get_access_statistics(
    conn: &Connection,
    limit: u32,
) -> Result<Vec<AccessStatistics>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT dst_domain,
            COUNT(dst_domain) as cnt
        FROM access_info
            WHERE dst_domain is not null
            GROUP BY dst_domain
            ORDER BY cnt desc
            LIMIT ?1
            ;",
    )?;
    stmt.query_map([limit], |row| {
        Ok(AccessStatistics {
            domain: row.get(0)?,
            count: row.get(1)?,
        })
    })
    .and_then(Iterator::collect)
}

#[allow(unused)]
fn test_db(db_path: &str) -> Result<()> {
    let conn = Connection::open(db_path).unwrap();
    conn.execute(
        "INSERT INTO access_info(id, time) VALUES(?1, ?2)",
        params!["1919-10-10 11:12:13"],
    )?;

    let mut stmt = conn.prepare(
        "SELECT cast(id as INT),
                                            time,
                                            cast(src_port as INT),
                                            src_ip,
                                            cast(dst_port as INT),
                                            dst_domain,
                                            state,
                                            protocol
                                        from access_info;",
    )?;
    let access_iter = stmt.query_map([], |row| {
        Ok(AccessInfo {
            id: row.get(0)?,
            time: row.get(1)?,
            src_port: row.get(2)?,
            src_ip: row.get(3)?,
            dst_port: row.get(4)?,
            dst_domain: row.get(5)?,
            state: row.get(6)?,
            protocol: row.get(7)?,
        })
    })?;

    for access in access_iter {
        println!("found access {:?}", access?);
    }
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
