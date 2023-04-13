use std::convert::TryInto;
use std::io;
use std::ops::{Div, Mul};


use log::{debug, error};

use crate::inverter_config::{InverterConfig, Mapping, ValueEnum};

#[derive(Debug, Clone)]
pub enum ConversionResult {
    StringResult(String),
    IntResult(i64),
    FloatResult(f64),
    BooleanResult(bool),
}

impl ConversionResult {
    pub fn apply_precision(&self, precision: &f64) -> Result<ConversionResult, String> {
        let result = self.try_resolve_precision(precision);
        if result.is_none() {
            let error_message = format!(
                "Could not apply precision {} for value {:?}",
                precision,
                self
            );
            return Err(error_message);
        }

        let mut result = result.unwrap();
        result = result.mul(100.0).trunc().div(100.0);

        debug!("Converted with precision {} to {}", precision, result);

        Ok(ConversionResult::FloatResult(result))
    }

    pub fn resolve_enum(&self, enum_mapping: &Vec<ValueEnum>) -> Result<ConversionResult, String> {
        let result = self.try_resolve_enum(enum_mapping);
        if result.is_none() {
            let error_message = format!(
                "Could not resolve enum {:?} for value {:?}",
                enum_mapping,
                self
            );
            return Err(error_message);
        }

        let result = result.unwrap();

        debug!("Converted to enum {}", result);

        Ok(ConversionResult::StringResult(result))
    }

    pub fn try_resolve_enum(&self, enum_mapping: &Vec<ValueEnum>) -> Option<String> {
        let value = self.to_string();

        let result = enum_mapping
            .iter()
            .find(|e| e.key == value)
            .map(|e| &e.value[..]);

        if result.is_none() {
            debug!(
                "Found no enum value for {} in mapping {:?}",
                value, enum_mapping
            );
            return None;
        }

        Some(String::from(result.unwrap()))
    }

    fn try_resolve_precision(&self, precision: &f64) -> Option<f64> {
        match self {
            ConversionResult::IntResult(conversion_result) => {
                let value: f64 = (*conversion_result) as f64;
                Some(value.mul(precision))
            }

            _ => {
                error!("Could not resolve precision for {:?}", self);
                None
            }
        }
    }
}

impl ToString for ConversionResult {
    fn to_string(&self) -> String {
        match self {
            ConversionResult::StringResult(value) => value.to_string(),
            ConversionResult::FloatResult(value) => value.to_string(),
            ConversionResult::BooleanResult(value) => value.to_string(),
            ConversionResult::IntResult(value) => value.to_string(),
        }
    }
}

pub trait ResolvePrecision {
    fn try_resolve_precision(&self, precision: f64) -> Option<f64>;
}

pub fn try_from_registers(mapping: &Mapping, registers: &Vec<u16>) -> Result<ConversionResult, String> {
    let conversion_result: Result<ConversionResult, String> = match mapping.data_type.as_str() {
        "string" | "hex" => {
            let human_readable: Result<String, String> =
                registers.try_into_human_readable(mapping.data_type.as_str());

            if human_readable.is_err() {
                let error_message = format!(
                    "Could not convert registers starting at {} to string",
                    mapping.register_address
                );
                return Err(error_message);
            }

            let human_readable = human_readable.unwrap();
            debug!("Converted to value(string): {}", human_readable);

            Ok(ConversionResult::StringResult(human_readable))
        }

        "u16" | "u32" | "i16" | "i32" => {
            let human_readable: Result<i64, String> =
                registers.try_into_human_readable(mapping.data_type.as_str());

            if human_readable.is_err() {
                let error_message = format!(
                    "Could not convert registers starting at {} to i64",
                    mapping.register_address
                );
                return Err(error_message);
            }

            let human_readable = human_readable.unwrap();
            debug!("Converted to value(i64): {}", human_readable);

            if mapping.statistic_type.is_some() && human_readable == 0 {
                let error_message = format!(
                    "Statistics mapping {:?} register returned 0. Skipping because we prevent unwanted overwriting of values",
                    mapping.register_address
                );
                return Err(error_message);
            }

            Ok(ConversionResult::IntResult(human_readable))
        }

        _ => {
            let error_message = format!(
                "No conversion mapping found for data_type {}",
                mapping.data_type
            );
            Err(error_message)
        }
    };

    let conversion_result = conversion_result?;

    // precision and enums should exclude each other
    if mapping.precision.is_some() {
        let precision = mapping.precision.as_ref().unwrap();
        let conversion_result = conversion_result.apply_precision(&precision)?;
    } else if mapping.value_enum.is_some() {
        let value_enum = mapping.value_enum.as_ref().unwrap();
        let conversion_result = conversion_result.resolve_enum(&value_enum)?;
    }

    Ok(conversion_result)
}

pub trait Convert<T> {
    fn try_into_human_readable(&self, data_type: &str) -> Result<T, String>;
}

impl Convert<String> for Vec<u16> {
    fn try_into_human_readable(&self, data_type: &str) -> Result<String, String> {
        match data_type {
            "string" => {
                let bytes: Vec<u8> = self
                    .iter()
                    .flat_map(|&r| r.to_be_bytes().to_vec())
                    .collect();

                Ok(String::from_utf8_lossy(&bytes).trim().to_owned())
            }

            "hex" => {
                let bytes: Vec<u8> = self
                    .iter()
                    .flat_map(|&r| r.to_be_bytes().to_vec())
                    .collect();

                Ok(hex::encode(bytes))
            }

            _ => Err(format!("No conversion specified for type {}", data_type)),
        }
    }
}

impl Convert<i64> for Vec<u16> {
    fn try_into_human_readable(&self, data_type: &str) -> Result<i64, String> {
        match data_type {
            "u32" => {
                let bytes: Vec<u8> = self
                    .iter()
                    .flat_map(|&r| r.to_be_bytes().to_vec())
                    .collect();

                let byte_slice = &bytes[..];
                let result = u32::from_be_bytes(byte_slice.try_into().unwrap()) as i64;
                Ok(result)
            }

            "i32" => {
                let bytes: Vec<u8> = self
                    .iter()
                    .flat_map(|&r| r.to_be_bytes().to_vec())
                    .collect();

                let byte_slice = &bytes[..];
                let result = i32::from_be_bytes(byte_slice.try_into().unwrap()) as i64;
                Ok(result)
            }

            "u16" | "i16" => Ok(self[0] as i64),
            _ => Err(format!("No conversion specified for type {}", data_type)),
        }
    }
}
