use log::{error, info};
use serde::{Deserialize, Serialize};
use std::io::BufReader;
use std::{
    env,
    fs::File,
    process::{self},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub application: ApplicationConfig,
    pub sever: SeverConfig,
    pub user: UserConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApplicationConfig {
    pub log_level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeverConfig {
    pub address: String,
    pub port: i64,
    pub ssl_tls: bool,
    pub ssl_verify_mode: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserConfig {
    pub token: String,
    pub nickname: String,
    pub main_channel: String,
    pub channels: Vec<String>,
}

pub fn load_config() -> Config {
    info!("Loading config...");
    let config_file_name = "config.json";
    let mut config_file = env::current_dir().unwrap();

    // config_file.push("config");
    config_file.push(config_file_name);

    info!("Config file path: {}", &config_file.display());
    let config_file = match File::open(&config_file) {
        Ok(file) => {
            info!("Successfully opened {}", &config_file.display());
            file
        }
        Err(e) => {
            error!("Failed to open {}, {}", &config_file.display(), e);
            process::exit(1);
        }
    };
    let config_file_data = BufReader::new(config_file);

    let config_data: Config = match serde_json::from_reader(config_file_data) {
        Ok(data) => {
            info!("Successfully parsed {}", &config_file_name);
            data
        }
        Err(e) => {
            error!("Failed to parse {}: {}", config_file_name, e);
            process::exit(1);
        }
    };
    config_data
}
