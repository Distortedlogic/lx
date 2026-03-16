use std::collections::HashMap;

pub struct Request {
    pub method: String,
    pub path: String,
    pub body: String,
    pub headers: HashMap<String, String>,
}

pub struct Response {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

pub struct Config {
    pub host: String,
    pub port: u16,
    pub debug: bool,
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
}

pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
}
