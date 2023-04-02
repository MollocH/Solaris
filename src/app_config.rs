use log::debug;
use std::fs::File;
use std::{env, fs};

use serde::{Deserialize, Serialize};


#[derive(Deserialize, Debug, Serialize)]
pub struct AppConfig {
    pub inverter: Inverter,
    pub influxdb2: Influxdb2,
    pub solaris: Solaris,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Inverter {
    pub inverter_address: String,
    pub inverter_port: u16,
    pub tcp_connect_timeout: u8,
    pub tcp_read_timeout: u8,
    pub inverter_modbus_uid: u8,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Influxdb2 {
    pub uri: String,
    pub org: String,
    pub bucket: String,
    pub token: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Solaris {
    pub read_frequency: u16,
}

pub fn read() -> AppConfig {
    let mut app_config_file = "config.yaml";

    let metadata = fs::metadata(app_config_file);

    if metadata.is_err() || !metadata.unwrap().is_file() {
        app_config_file = "config.yaml.dist";
    }

    let f =
        File::open(app_config_file).unwrap_or_else(|_| panic!("Could not open file '{}'", app_config_file));
    let mut app_config: AppConfig = serde_yaml::from_reader(f)
        .unwrap_or_else(|_| panic!("Could not parse yaml file '{}'", app_config_file));

    overwrite_app_config_with_environment(&mut app_config);

    debug!(
        "final config: {}",
        serde_json::to_string_pretty(&app_config).unwrap()
    );

    app_config
}

fn overwrite_app_config_with_environment(app_config: &mut AppConfig) {
    app_config.inverter.inverter_address = parse_env_var(
        "SOLARIS_INVERTER_ADDRESS",
        app_config.inverter.inverter_address.clone(),
    );
    app_config.inverter.inverter_port = parse_env_var(
        "SOLARIS_INVERTER_PORT",
        app_config.inverter.inverter_port,
    );
    app_config.inverter.tcp_read_timeout = parse_env_var(
        "SOLARIS_INVERTER_TCP_READ_TIMEOUT",
        app_config.inverter.tcp_read_timeout,
    );
    app_config.inverter.tcp_connect_timeout = parse_env_var(
        "SOLARIS_INVERTER_TCP_CONNECT_TIMEOUT",
        app_config.inverter.tcp_connect_timeout,
    );
    app_config.inverter.inverter_modbus_uid = parse_env_var(
        "SOLARIS_INVERTER_MODBUS_UID",
        app_config.inverter.inverter_modbus_uid,
    );
    app_config.influxdb2.uri =
        parse_env_var("SOLARIS_INFLUXDB2_URI", app_config.influxdb2.uri.clone());
    app_config.influxdb2.org =
        parse_env_var("SOLARIS_INFLUXDB2_ORG", app_config.influxdb2.org.clone());
    app_config.influxdb2.bucket = parse_env_var(
        "SOLARIS_INFLUXDB2_BUCKET",
        app_config.influxdb2.bucket.clone(),
    );
    app_config.influxdb2.token = parse_env_var(
        "SOLARIS_INFLUXDB2_TOKEN",
        app_config.influxdb2.token.clone(),
    );
    app_config.solaris.read_frequency = parse_env_var(
        "SOLARIS_READ_FREQUENCY",
        app_config.solaris.read_frequency,
    );
}

fn parse_env_var<T: std::str::FromStr>(name: &str, default: T) -> T {
    match env::var(name) {
        Ok(val) => {
            debug!("Found {} env with value {}", name, val);
            val.parse().unwrap_or(default)
        }
        Err(_) => default,
    }
}
