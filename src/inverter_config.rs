use serde::Deserialize;
use std::fs::File;

#[derive(Deserialize, Debug)]
pub struct InverterConfig {
    pub inverter_slug: String,
    pub mappings: Vec<Mapping>,
    pub statistic_mappings: Vec<StatisticMapping>
}

#[derive(Deserialize, Debug, Clone)]
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
    #[serde(default)]
    pub statistic_type: Option<StatisticType>,
    #[serde(default)]
    pub statistic_pointer: Option<u32>
}

#[derive(Deserialize, Debug, Clone)]
pub struct StatisticMapping {
    pub name: String,
    pub register_address: u16,
    #[serde(default = "default_register_length")]
    pub length: u16,
    pub data_type: String,
    #[serde(default)]
    pub precision: Option<f64>,
    #[serde(default)]
    pub statistic_type: Option<StatisticType>
}

fn default_register_length() -> u16 {
    1
}

#[derive(Deserialize, Debug, Clone)]
pub struct ValueEnum {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Debug, Clone)]
pub enum StatisticType {
    Daily,
    Monthly
}

pub fn read(config_file: &String) -> InverterConfig {
    let f = File::open(config_file).unwrap_or_else(|_| panic!("Could not open file '{}'", &config_file));
    let result = serde_yaml::from_reader(f);
    if let Err(e) = result {
        panic!("Could not parse yaml file '{}'. Error was '{}'", &config_file, e)
    }
    result.unwrap()
}
