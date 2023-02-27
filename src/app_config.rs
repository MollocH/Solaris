use std::fs::File;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub inverter: Inverter,
    pub influxdb2: Influxdb2,
    pub solaris: Solaris,
}

#[derive(Deserialize, Debug)]
pub struct Inverter {
    pub inverter_address: String,
    pub inverter_port: u16,
    pub tcp_connect_timeout: u8,
    pub tcp_read_timeout: u8,
    pub inverter_modbus_uid: u8,
}

#[derive(Deserialize, Debug)]
pub struct Influxdb2 {
    pub uri: String,
    pub org: String,
    pub bucket: String,
    pub token: String,
}

#[derive(Deserialize, Debug)]
pub struct Solaris {
    pub read_frequency: u16,
}

pub fn read() -> AppConfig {
    let app_config_file = "config.yaml";
    let f =
        File::open(app_config_file).expect(&format!("Could not open file '{}'", app_config_file));
    return serde_yaml::from_reader(f)
        .expect(&format!("Could not parse yaml file '{}'", app_config_file));
}
