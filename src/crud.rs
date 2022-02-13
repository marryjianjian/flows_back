use crate::model::{AccessInfo, AccessStatistics, DayStatistics};
use rusqlite::{params, Connection, Result};
use regex::Regex;
use lazy_static::lazy_static;

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

lazy_static! {
    static ref DATE_QUERY_FORMAT: Regex = Regex::new(
        r"(?x)
        ([0-9]{4})-([0-9]{1,2})-([0-9]{1,2}) # 1, 2, 3 yyyy-mm-dd
        "
    )
    .unwrap();
}

pub fn parse_date(date_str: &str) -> Option<(i32, i32, i32)> {
    match DATE_QUERY_FORMAT.captures(date_str) {
        Some(caps) => {
            let year = caps.get(1).map_or(None, |m| Some(m.as_str().parse::<i32>().unwrap()))?;
            let month = caps.get(2).map_or(None, |m| Some(m.as_str().parse::<i32>().unwrap()))?;
            let day = caps.get(3).map_or(None, |m| Some(m.as_str().parse::<i32>().unwrap()))?;
            Some((year, month, day))
        },
        None => {
            None
        }
    }
}

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
pub async fn get_top_10_domain_statistics(
    conn: &Pool,
    limit: u32,
) -> Result<Vec<AccessStatistics>, rusqlite::Error> {
    let mut conn = conn.get().expect("get connection from pool error");
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
pub async fn get_last_7_days_statistics(
    conn: &Pool,
) -> Result<Vec<DayStatistics>, rusqlite::Error> {
    let mut conn = conn.get().expect("get connection from pool error");
    let tx = conn.transaction()?;
    let mut res: Vec<DayStatistics> = Vec::new();

    // stmt need Droped first
    {
        let mut stmt = tx.prepare(
            "SELECT time, count(time)
                FROM access_info
                    WHERE instr(time, date('now', ?1))",
        )?;
        for i in 0..6 {
            let day = format!("-{} day", i);
            stmt.query_row(params![day], |row| {
                match row.get(0) {
                    Ok(date_name) => {
                        res.push(DayStatistics {
                            date_name: date_name,
                            domain_count: row.get(1)?,
                        });
                    }
                    Err(err) => {
                        println! {"ignored date({}) err: {}", day, err};
                        //TODO: log here
                    }
                }
                Ok(())
            })?;
        }
    }
    tx.commit()?;
    Ok(res)
}

#[allow(unused)]
pub async fn get_days_statistics(
    conn: &Pool,
    date_strs: Vec<&str>
) -> Result<Vec<DayStatistics>, rusqlite::Error> {
    let mut conn = conn.get().expect("get connection from pool error");
    let tx = conn.transaction()?;
    let mut res: Vec<DayStatistics> = Vec::new();

    // stmt need Droped first
    {
        let mut stmt = tx.prepare(
            "SELECT time, count(time)
                FROM access_info
                    WHERE instr(time, ?1)",
        )?;
        for date in date_strs.into_iter() {
            match parse_date(date) {
                Some(_) => {},
                None => {
                    res.push(DayStatistics {
                        date_name: date.to_string(),
                        domain_count: 0,
                    });
                }
            }
            stmt.query_row(params![date], |row| {
                match row.get(0) {
                    Ok(date_name) => {
                        res.push(DayStatistics {
                            date_name: date_name,
                            domain_count: row.get(1)?,
                        });
                    }
                    Err(err) => {
                        println! {"ignored date({}) err: {}", date, err};
                        //TODO: log here
                    }
                }
                Ok(())
            })?;
        }
    }
    tx.commit()?;
    Ok(res)
}

#[allow(unused)]
pub fn test_db(db_path: &str) -> Result<()> {
    let conn = Connection::open(db_path).unwrap();
    conn.execute(
        "INSERT INTO access_info(time) VALUES(?1)",
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

#[cfg(test)]
mod tests {
    use super::*;

    const DB_PATH: &str = "/home/jian/code/flows_back/src/db/t.db";

    #[test]
    fn test_db_() {
        assert_eq!(test_db(DB_PATH), Ok(()));
    }

    #[test]
    fn test_parse_date() {
        let cases:Vec<(&str, bool, i32, i32, i32)> = vec![
            ("1212-12-01", true, 1212, 12, 1),
            ("2021-12-01", true, 2021, 12, 1),
            ("12-01-1234", false, 0, 0, 0),
            ("asdf", false, 0, 0, 0),
        ];

        for c in cases.into_iter() {
            match parse_date(c.0) {
                Some(ymd) => {
                    assert_eq!(c.2, ymd.0);
                    assert_eq!(c.3, ymd.1);
                    assert_eq!(c.4, ymd.2)
                },
                None => {
                    assert_eq!(c.1, false);
                }
            }
        }
    }
}
