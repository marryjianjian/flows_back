use crate::model::{AccessInfo, AccessStatistics};
use rusqlite::{params, Connection, Result};

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

#[allow(unused)]
pub async fn update_database(conn: &Pool, access_infos: &Vec<AccessInfo>) -> Result<usize> {
    let mut conn = conn.get().expect("get connection from pool error");
    let tx = conn.transaction()?;
    // stmt need Droped first
    {
        let mut stmt = tx.prepare(
            "INSERT OR IGNORE INTO
                access_info(time, src_port, src_ip, dst_port, dst_domain, state, protocol, tag)
                VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        for access_info in access_infos {
            stmt.execute(params![
                access_info.time,
                access_info.src_port,
                access_info.src_ip,
                access_info.dst_port,
                access_info.dst_domain,
                access_info.state,
                access_info.protocol,
                access_info.tag,
            ])?;
        }
    }
    tx.commit()?;
    Ok(access_infos.len())
}

#[allow(unused)]
pub async fn get_access_statistics(
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
pub fn test_db(db_path: &str) -> Result<()> {
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
                protocol,
                tag
            FROM access_info;",
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
            tag: row.get(8)?,
        })
    })?;

    for access in access_iter {
        println!("found access {:?}", access?);
    }
    Ok(())
}
