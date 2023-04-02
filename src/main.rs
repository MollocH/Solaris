use std::env;
use std::time::Duration;

use convert_case::{Case, Casing};
use futures::stream;
use influxdb2::models::DataPoint;
use log::{debug, error, info};
use modbus::{tcp, Client, Transport};

use convert::Convert;

use crate::convert::{ConversionResult};

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
            debug!("-------------------------------------------------------------------");
            debug!("Processing mapping: {:?}", mapping);
            let registers =
                modbus_client.read_input_registers(mapping.register_address - 1, mapping.length);

            if registers.is_err() {
                error!(
                    "Could not read input registers at address {} with length {}",
                    mapping.register_address, mapping.length
                );
                continue;
            }

            let registers = registers.unwrap();

            let mut conversion_result = match mapping.data_type.as_str() {
                "string" | "hex" => {
                    let human_readable: Result<String, String> =
                        registers.try_into_human_readable(mapping.data_type.as_str());

                    if human_readable.is_err() {
                        error!(
                            "Could not convert registers starting at {} to string",
                            mapping.register_address
                        );
                        continue;
                    }

                    let human_readable = human_readable.unwrap();
                    debug!("Converted to value(string): {}", human_readable);

                    ConversionResult::StringResult(human_readable)
                }

                "u16" | "u32" | "i16" | "i32" => {
                    let human_readable: Result<i64, String> =
                        registers.try_into_human_readable(mapping.data_type.as_str());

                    if human_readable.is_err() {
                        error!(
                            "Could not convert registers starting at {} to i64",
                            mapping.register_address
                        );
                        continue;
                    }

                    let human_readable = human_readable.unwrap();
                    debug!("Converted to value(i64): {}", human_readable);

                    ConversionResult::IntResult(human_readable)
                }

                _ => {
                    error!(
                        "No conversion mapping found for data_type {}",
                        mapping.data_type
                    );
                    continue;
                }
            };

            // precision and enums should exclude each other
            if mapping.precision.is_some() {
                let precision = mapping.precision.as_ref().unwrap();
                let result = conversion_result.try_resolve_precision(&precision);
                if result.is_none() {
                    error!(
                        "Could not apply precision {:?} for value {:?}",
                        mapping.precision.unwrap(),
                        conversion_result
                    );
                    continue;
                }

                let result = result.unwrap();

                debug!("Converted with precision {} to {}", precision, result);

                conversion_result = ConversionResult::FloatResult(result);
            } else if mapping.value_enum.is_some() {
                let enum_mapping = mapping.value_enum.as_ref().unwrap();
                let result = conversion_result.try_resolve_enum(&enum_mapping);
                if result.is_none() {
                    error!(
                        "Could not resolve enum {:?} for value {:?}",
                        enum_mapping,
                        conversion_result
                    );
                    continue;
                }

                let result = result.unwrap();

                debug!("Converted to enum {}", result);

                conversion_result = ConversionResult::StringResult(result);
            }

            let datapoint_builder = DataPoint::builder(&inverter_config.inverter_slug.to_case(Case::Snake))
                .tag("inverter", &inverter_config.inverter_slug)
                .tag("ip_address", &app_config.inverter.inverter_address);

            let datapoint = match conversion_result {
                ConversionResult::StringResult(value) => datapoint_builder.field(mapping.name.to_case(Case::Snake), value).build(),
                ConversionResult::IntResult(value) => datapoint_builder.field(mapping.name.to_case(Case::Snake), value).build(),
                ConversionResult::FloatResult(value) => datapoint_builder.field(mapping.name.to_case(Case::Snake), value).build(),
                ConversionResult::BooleanResult(value) => datapoint_builder.field(mapping.name.to_case(Case::Snake), value).build(),
            };

            if datapoint.is_err() {
                error!(
                    "Datapoint could not be created for mapping {:?}. Error was {:?}",
                    mapping,
                    datapoint.err().unwrap().to_string()
                );
                continue;
            }

            datapoints.push(datapoint.unwrap());
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
