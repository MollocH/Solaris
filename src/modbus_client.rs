use std::time::Duration;

use modbus::{Client, tcp, Transport};
use crate::app_config::Inverter;

use super::app_config::AppConfig;
use super::inverter_config::Mapping;

pub struct SolarisModbusClient {
    client: Transport
}

pub trait ModbusClient {
    fn new(inverter_config: &Inverter) -> Self;
    fn read_register(&mut self, mapping: &Mapping) -> Vec<u16>;
}

impl ModbusClient for SolarisModbusClient {
    fn new(inverter_config: &Inverter) -> Self {
        let cfg = tcp::Config {
            tcp_connect_timeout: Some(Duration::new(u64::from(inverter_config.tcp_connect_timeout), 0)),
            tcp_port: inverter_config.inverter_port,
            tcp_read_timeout: Some(Duration::new(u64::from(inverter_config.tcp_read_timeout), 0)),
            tcp_write_timeout: None,
            modbus_uid: inverter_config.inverter_modbus_uid
        };

        let client = tcp::Transport::new_with_cfg(inverter_config.inverter_address.as_str(), cfg).unwrap();
        return SolarisModbusClient { client };
    }

    fn read_register(&mut self, mapping: &Mapping) -> Vec<u16> {
        let start_register_address = mapping.register_address - 1;
        let registers = self.client.read_input_registers(start_register_address, mapping.length).expect("IO Error");
        return registers;
    }
}
