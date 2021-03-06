use crate::model::AccessInfo;
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use std::io::BufRead;
use std::{fs, io};

#[allow(dead_code)]
pub fn read_access_info_fron_line(line: &str) -> Option<AccessInfo> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?x)
            (?:access_log_([a-z0-9]+)\s)?          # 1, tag
            ([0-9]{4})/([0-9]{1,2})/([0-9]{1,2})\s # 2,3,4, yyyy-mm-dd
            ([0-9]{2}:[0-9]{2}:[0-9]{2})\s         # 5, hh:mm:ss
            ((?:\d{1,3}\.){3}\d{1,3}):             # 6, src_ip xxx.xxx.xxx.xxx
            (\d{1,5})\s                            # 7, src_port xxxxx
            (accepted|rejected)\s                  # 8, state
            (?:(tcp|udp):                          # 9, protocol
            ([0-9A-Za-z\-.]+):                     # 10, dst_domain
            (\d{1,5})|.+)                          # 11, dst_port
            "
        )
        .unwrap();
    }
    match RE.captures(line) {
        Some(caps) => {
            let tag = caps.get(1).map_or(None, |m| Some(m.as_str().to_string()));
            let year = caps.get(2).map_or("", |m| m.as_str());
            let month = caps.get(3).map_or("", |m| m.as_str());
            let day = caps.get(4).map_or("", |m| m.as_str());
            let hms = caps.get(5).map_or("", |m| m.as_str());
            let src_ip = caps.get(6).map_or(None, |m| Some(m.as_str().to_string()));
            let state = caps.get(8).map_or(None, |m| Some(m.as_str().to_string()));
            let protocol = caps.get(9).map_or(None, |m| Some(m.as_str().to_string()));
            let dst_domain = caps.get(10).map_or(None, |m| Some(m.as_str().to_string()));
            let src_port = caps
                .get(7)
                .map_or(None, |m| Some(m.as_str().parse::<u32>().unwrap()));
            let dst_port = caps
                .get(11)
                .map_or(None, |m| Some(m.as_str().parse::<u32>().unwrap()));

            return Some(AccessInfo {
                id: 0,
                time: format!("{}-{}-{} {}", year, month, day, hms),
                src_port: src_port,
                src_ip: src_ip,
                dst_port: dst_port,
                dst_domain: dst_domain,
                state: state,
                protocol: protocol,
                tag: tag,
            });
        }
        None => {
            warn!("captured nothing from '{:?}'", line);
            return None;
        }
    }
}

#[allow(dead_code)]
pub fn read_access_info_from_file(log_path: &str) -> io::Result<Vec<AccessInfo>> {
    let f = fs::File::open(log_path)?;
    let reader = io::BufReader::new(f);

    let mut res: Vec<AccessInfo> = Vec::new();

    for line in reader.lines() {
        match line {
            Ok(l) => match read_access_info_fron_line(&l) {
                Some(access_info) => {
                    res.push(access_info);
                }
                None => {}
            },
            Err(_) => {}
        }
    }

    Ok(res)
}
