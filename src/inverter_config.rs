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
    #[serde(default = "default_register_length")]
    pub length: u16,
    pub data_type: String,
    #[serde(default)]
    pub precision: Option<f64>,
    #[serde(default)]
    pub value_enum: Option<Vec<ValueEnum>>,
}

fn default_register_length() -> u16 {
    1
}

#[derive(Deserialize, Debug)]
pub struct ValueEnum {
    pub key: String,
    pub value: String,
}

pub fn read(config_file: &String) -> InverterConfig {
    let f = File::open(config_file).unwrap_or_else(|_| panic!("Could not open file '{}'", &config_file));
    serde_yaml::from_reader(f)
        .unwrap_or_else(|_| panic!("Could not parse yaml file '{}'", &config_file))
}
