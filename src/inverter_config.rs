use serde::Deserialize;
use std::fs::File;

#[derive(Deserialize, Debug)]
pub struct InverterConfig {
    pub inverter_slug: String,
    pub mappings: Vec<Mapping>,
}

#[derive(Deserialize, Debug)]
pub struct Mapping {
    pub name: String,
    pub register_address: u16,
    pub length: u16,
    pub data_type: String,
}

pub fn read(config_file: &String) -> InverterConfig {
    let f = File::open(config_file).expect(&format!("Could not open file '{}'", &config_file));
    return serde_yaml::from_reader(f)
        .expect(&format!("Could not parse yaml file '{}'", &config_file));
}
