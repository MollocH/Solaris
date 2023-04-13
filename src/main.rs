use std::{env, time};
use std::ops::{Add, Sub};
use std::time::Duration;

use convert_case::{Case, Casing};
use futures::stream;
use influxdb2::models::DataPoint;
use log::{debug, error, info};
use modbus::{tcp, Client, Transport};

use chrono::prelude::*;

use crate::convert::{ConversionResult, try_from_registers};
use crate::inverter_config::{Mapping, StatisticType};

mod app_config;
mod convert;
mod inverter_config;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let config_file = &args[1];

    let app_config = app_config::read();

    let inverter_config = inverter_config::read(config_file);
    let influxdb_client = get_influxdb2_client(&app_config.influxdb2);

    info!("-------------------------------------------------------------------");
    info!("Starting read cycle ...");

    loop {
        let mut modbus_client = get_modbus_client(&app_config.inverter);
        let mut datapoints: Vec<DataPoint> = Vec::new();

        let mut mappings = inverter_config.mappings.clone();

        for mapping in &inverter_config.statistic_mappings {
            match mapping.statistic_type.as_ref().unwrap() {
                StatisticType::Daily => {
                    let now = Local::now();

                    let seconds_to_midnight = 86400.sub(now.num_seconds_from_midnight());

                    // skip close to midnight to prevent a race condition.
                    // register will reset after a month change so we want to prevent overwriting old values with zeros
                    if seconds_to_midnight < 30 {
                        debug!("Skipping statistic because only {} seconds until midnight", seconds_to_midnight);
                        continue;
                    }

                    let current_day = now.day();

                    for i in 1..=current_day {
                        let start_register_address = mapping.register_address.clone();
                        let register_address = start_register_address.add(i.sub(1) as u16);

                        mappings.push(Mapping {
                            name: mapping.name.clone(),
                            register_address,
                            length: mapping.length.clone(),
                            data_type: mapping.data_type.clone(),
                            precision: mapping.precision.clone(),
                            value_enum: None,
                            statistic_type: mapping.statistic_type.clone(),
                            statistic_pointer: Some(i)
                        })
                    }
                }
                _ => todo!("")
            };
        }

        for mapping in &mappings {
            debug!("-------------------------------------------------------------------");
            debug!("Processing mapping: {:?}", mapping);

            let registers = match modbus_client.read_input_registers(mapping.register_address - 1, mapping.length) {
                Ok(registers) => registers,
                Err(_) => {
                    error!(
                        "Could not read input registers at address {} with length {}",
                        mapping.register_address, mapping.length
                    );
                    continue;
                }
            };

            debug!("Registers value: {:?}", registers);

            let conversion_result = match try_from_registers(&mapping, &registers){
                Ok(conversion_result) => conversion_result,
                Err(e) => {
                    error!("{}", e);
                    continue
                }
            };

            let datapoint = match build_datapoint(&conversion_result, &mapping, &inverter_config.inverter_slug, &app_config.inverter.inverter_address) {
                Ok(datapoint) => datapoint,
                Err(e) => {
                    error!("{}", e);
                    continue
                }
            };

            datapoints.push(datapoint);
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

fn build_datapoint(conversion_result: &ConversionResult, mapping: &Mapping, inverter_slug: &String, inverter_address: &String) -> Result<DataPoint, String> {
    let mut datapoint_builder = DataPoint::builder(inverter_slug.to_case(Case::Snake))
        .tag("inverter", inverter_slug)
        .tag("ip_address", inverter_address);

    if mapping.statistic_pointer.is_some() {
        match mapping.statistic_type.as_ref().unwrap() {
            StatisticType::Daily => {
                let day_of_month = mapping.statistic_pointer.unwrap();
                let mut start_of_day = chrono::Local::now();
                start_of_day = start_of_day.with_day(day_of_month).unwrap();
                start_of_day = start_of_day.with_hour(0).unwrap();
                start_of_day = start_of_day.with_minute(0).unwrap();
                start_of_day = start_of_day.with_second(0).unwrap();
                start_of_day = start_of_day.with_nanosecond(0).unwrap();

                debug!("{:?}", start_of_day.timestamp_nanos());

                datapoint_builder = datapoint_builder.timestamp(start_of_day.timestamp_nanos());
            }
            _ => todo!("")
        };
    }

    let datapoint = match conversion_result {
        ConversionResult::StringResult(value) => datapoint_builder.field(mapping.name.to_case(Case::Snake), value.clone()).build(),
        ConversionResult::IntResult(value) => datapoint_builder.field(mapping.name.to_case(Case::Snake), value.clone()).build(),
        ConversionResult::FloatResult(value) => datapoint_builder.field(mapping.name.to_case(Case::Snake), value.clone()).build(),
        ConversionResult::BooleanResult(value) => datapoint_builder.field(mapping.name.to_case(Case::Snake), value.clone()).build(),
    };

    if datapoint.is_err() {
        let error_message = format!(
            "Datapoint could not be created for mapping {:?}. Error was {:?}",
            mapping,
            datapoint.err().unwrap().to_string()
        );
        return Err(error_message);
    }

    Ok(datapoint.unwrap())
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
