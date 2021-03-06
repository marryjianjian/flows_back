use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct AccessInfo {
    pub id: u64,
    pub time: String,
    pub src_port: Option<u32>,
    pub src_ip: Option<String>,
    pub dst_port: Option<u32>,
    pub dst_domain: Option<String>,
    pub state: Option<String>,
    pub protocol: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessStatistics {
    pub domain: String,
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DayStatistics {
    pub date_name: String,
    pub domain_count: u64,
}
