use std::env;

use crate::inverter_config::ValueEnum;
use convert::Convert;
use convert_case::{Case, Casing};
use futures::stream;
use influxdb2::models::DataPoint;
use log::{debug, error, info};
use modbus::{tcp, Client, Transport};
use std::time::Duration;

mod app_config;
mod convert;
mod inverter_config;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let config_file = &args[1];

    let app_config = app_config::read();

    let inverter_config = inverter_config::read(&config_file);
    let influxdb_client = get_influxdb2_client(&app_config.influxdb2);

    info!("-------------------------------------------------------------------");
    info!("Starting read cycle ...");

    loop {
        let mut modbus_client = get_modbus_client(&app_config.inverter);
        let mut datapoints: Vec<DataPoint> = Vec::new();
        for mapping in &inverter_config.mappings {
            let registers =
                modbus_client.read_input_registers(mapping.register_address - 1, mapping.length);

            if registers.is_err() {
                error!(
                    "Could not read register at address {} (-1)",
                    mapping.register_address
                );
                continue;
            }

            let registers = registers.unwrap();

            let datapoint_builder =
                DataPoint::builder(&inverter_config.inverter_slug.to_case(Case::Snake))
                    .tag("inverter", &inverter_config.inverter_slug)
                    .tag("ip_address", &app_config.inverter.inverter_address);

            match mapping.data_type.as_str() {
                "string" => {
                    let result = registers.convert_to_string();
                    debug!(
                        "Register name {} has been read as type {} with value {}",
                        mapping.name, mapping.data_type, result
                    );
                    let datapoint = datapoint_builder
                        .field(mapping.name.to_case(Case::Snake), result.clone())
                        .build()
                        .unwrap();
                    datapoints.push(datapoint);
                }
                "hex" => {
                    let result = registers.convert_to_hex();
                    debug!(
                        "Register name {} has been read as type {} with value {}",
                        mapping.name, mapping.data_type, result
                    );
                    let datapoint = datapoint_builder
                        .field(mapping.name.to_case(Case::Snake), result.clone())
                        .build()
                        .unwrap();
                    datapoints.push(datapoint);
                }
                "float" => {
                    let result = registers.convert_to_float(&mapping.precision);
                    debug!(
                        "Register name {} has been read as type {} with value {}",
                        mapping.name, mapping.data_type, result
                    );
                    let datapoint = datapoint_builder
                        .field(mapping.name.to_case(Case::Snake), result)
                        .build()
                        .unwrap();
                    datapoints.push(datapoint);
                }
                "u32" => {
                    let result = i64::from(registers.convert_to_u32());
                    debug!(
                        "Register name {} has been read as type {} with value {}",
                        mapping.name, mapping.data_type, result
                    );

                    if mapping.value_enum.is_some() {
                        let old_result = result.clone();
                        let value_enum = &mapping.value_enum.as_ref().unwrap();
                        let result = resolve_enum_value(&result, &value_enum);
                        debug!(
                            "Register name {} has an enum mapping which converted {} to {}",
                            mapping.name, old_result, result
                        );
                    }

                    let datapoint = datapoint_builder
                        .field(mapping.name.to_case(Case::Snake), result)
                        .build()
                        .unwrap();
                    datapoints.push(datapoint);
                }
                _ => {
                    debug!("{:?}", registers);
                    error!(
                        "register_type {} has no defined conversion. Skipping",
                        mapping.data_type
                    );
                    continue;
                }
            }
        }
        modbus_client.close().unwrap();

        let influxdb2_write_result = influxdb_client
            .write(
                app_config.influxdb2.bucket.as_str(),
                stream::iter(datapoints),
            )
            .await;

        if influxdb2_write_result.is_err() {
            error!("{}", influxdb2_write_result.unwrap_err().to_string());
        }

        info!(
            "finished cycle. Waiting for {} seconds ...",
            app_config.solaris.read_frequency
        );
        info!("-------------------------------------------------------------------");

        tokio::time::sleep(Duration::from_secs(
            app_config.solaris.read_frequency.into(),
        ))
        .await;
    }
}

fn resolve_enum_value<T: PartialEq + std::fmt::Debug>(
    key: T,
    enum_mapping: &Vec<ValueEnum>,
) -> String {
    let result = enum_mapping
        .iter()
        .find(|e| e.key == format!("{:?}", key))
        .map(|e| &e.value[..]);

    if result.is_none() {
        // TODO: Handle
    }

    result.unwrap().to_string()
}

fn get_influxdb2_client(influxdb2: &app_config::Influxdb2) -> influxdb2::Client {
    influxdb2::Client::new(&influxdb2.uri, &influxdb2.org, &influxdb2.token)
}

fn get_modbus_client(inverter_config: &app_config::Inverter) -> Transport {
    let cfg = tcp::Config {
        tcp_connect_timeout: Some(Duration::new(
            u64::from(inverter_config.tcp_connect_timeout),
            0,
        )),
        tcp_port: inverter_config.inverter_port,
        tcp_read_timeout: Some(Duration::new(
            u64::from(inverter_config.tcp_read_timeout),
            0,
        )),
        tcp_write_timeout: None,
        modbus_uid: inverter_config.inverter_modbus_uid,
    };

    Transport::new_with_cfg(inverter_config.inverter_address.as_str(), cfg).unwrap()
}
