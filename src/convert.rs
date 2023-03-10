use hex;
use influxdb2::models::FieldValue;
use log::{debug, error, info};
use std::convert::TryInto;
use std::error::Error;
use std::fs::read_to_string;
use std::ops::{Div, Mul};
use crate::inverter_config::ValueEnum;

#[derive(Debug)]
pub enum ConversionResult {
    StringResult(String),
    IntResult(i64),
    FloatResult(f64),
    BooleanResult(bool)
}

pub trait ResolveEnumValue {
    fn try_resolve_enum(&self, enum_mapping: Vec<ValueEnum>) -> Option<String> {
        let value: String = (*self).into();

        let result = enum_mapping
            .iter()
            .find(|e| e.key == value)
            .map(|e| &e.value[..]);

        if result.is_none() {
            debug!("Found no enum value for {} in mapping {:?}", value, enum_mapping);
            return None
        }

        Some(String::from(result))
    }
}

impl ResolveEnumValue for ConversionResult{}

pub trait ResolvePrecision {
    fn try_resolve_precision(&self, precision: f64) -> Option<f64>;
}

impl ResolvePrecision for ConversionResult {
    fn try_resolve_precision(&self, precision: f64) -> Option<f64> {
        match self {
            ConversionResult::IntResult(conversion_result) => {
                let value: f64 = conversion_result.into();
                some(value.div(precision))
            }

            _ => {
                error!("Could not resolve precision for {:?}", self);
                None
            }
        }

    }
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

                Ok(hex::encode(&bytes))
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
