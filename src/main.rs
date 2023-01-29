use std::env;
use std::time::SystemTime;

use convert::Convert;
use convert_case::{Case, Casing};
use influxdb2::models::DataPoint;
use influxdb2_client::Influxdb2Client;
use influxdb2_client::SolarisInfluxdb2Client;
use modbus_client::ModbusClient;
use modbus_client::SolarisModbusClient;

mod inverter_config;
mod modbus_client;
mod app_config;
mod convert;
mod influxdb2_client;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let config_file = &args[1];

    let config = app_config::read();

    let inverter_config = inverter_config::read(&config_file);

    let mut modbus_client: SolarisModbusClient = ModbusClient::new(&config.inverter);
    let mut influx_client: SolarisInfluxdb2Client = Influxdb2Client::new(&config.influxdb2);

    let mut datapoints: Vec<DataPoint> = Vec::new();

    for mapping in inverter_config.mappings {
        let registers = modbus_client.read_register(&mapping);

        let mut datapoint_builder = DataPoint::builder(&inverter_config.inverter_slug.to_case(Case::Snake))
            .tag("inverter", &inverter_config.inverter_slug)
            .tag("ip_address", &config.inverter.inverter_address);

        match mapping.data_type.as_str() {
            "string" => {
                let result = registers.convert_to_string();
                datapoints.push(datapoint_builder.field(mapping.name.to_case(Case::Snake), result).build().unwrap());
            }
            "hex" => {
                let result = registers.convert_to_hex();
                datapoints.push(datapoint_builder.field(mapping.name.to_case(Case::Snake), result).build().unwrap());
            }
            "decimal10" => {
                let result = registers.convert_to_decimal10();
                datapoints.push(datapoint_builder.field(mapping.name.to_case(Case::Snake), result).build().unwrap());
            }
            _ => {
                panic!("register_type {} has no defined conversion", mapping.data_type)
            }
        }
    }

    influx_client.write(datapoints).await;
}
