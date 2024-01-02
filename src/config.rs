use std::fs;
use serde::{Serialize, Deserialize};
use serde_yaml::from_str;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub accepted_users: Vec<String>,
    pub number_of_threads: i32
}

pub fn read_config(path: &str) -> serde_yaml::Result<Config> {
    let contents = fs::read_to_string(path)
        .expect("Something went wrong reading the file");
    return from_str::<Config>(&contents)
}