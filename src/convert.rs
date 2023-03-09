use hex;
use influxdb2::models::FieldValue;
use log::{debug, error, info};
use std::convert::TryInto;
use std::error::Error;
use std::fs::read_to_string;
use std::ops::Mul;

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

            "u16" => Ok(self[0] as i64),
            "i16" => Ok(self[0] as i64),
            _ => Err(format!("No conversion specified for type {}", data_type)),
        }
    }
}

// impl Convert<f64> for Vec<u16> {
//     fn try_into_human_readable(&self, data_type: &str) -> Result<f64, String> {
//         match data_type {
//             "float" => {
//                 let bytes: Vec<u8> = self
//                     .iter()
//                     .flat_map(|&r| r.to_be_bytes().to_vec())
//                     .collect();
//
//                 let byte_slice = &bytes[..];
//                 let result = f64::from_be_bytes(byte_slice.try_into().unwrap());
//                 Ok(result)
//             }
//
//             _ => Err(format!("No conversion specified for type {}", data_type)),
//         }
//     }
// }
//
// impl Convert<bool> for Vec<u16> {
//     fn try_into_human_readable(&self, data_type: &str) -> Result<bool, String> {
//         match data_type {
//             "bool" => Ok(self[0] > 0),
//             _ => Err(format!("No conversion specified for type {}", data_type)),
//         }
//     }
// }
