use hex;
use std::ops::{Mul};

pub trait Convert {
    fn convert_to_string(&self) -> String;
    fn convert_to_hex(&self) -> String;
    fn convert_to_float(&self, precision: &f64) -> f64;
    fn convert_to_u32(&self) -> u32;
}

impl Convert for Vec<u16> {
    fn convert_to_string(&self) -> String {
        let bytes: Vec<u8> = self
            .iter()
            .flat_map(|&r| r.to_be_bytes().to_vec())
            .collect();

        String::from_utf8_lossy(&bytes).trim().to_owned()
    }

    fn convert_to_hex(&self) -> String {
        let bytes: Vec<u8> = self
            .iter()
            .flat_map(|&r| r.to_be_bytes().to_vec())
            .collect();
        hex::encode(&bytes)
    }

    fn convert_to_float(&self, precision: &f64) -> f64 {
        f64::from(self[0]).mul(precision)
    }

    fn convert_to_u32(&self) -> u32 {
        if self.len() == 1 {
            return u32::from(self[0]);
        }

        let bytes: [u8; 4] = self
            .iter()
            .map(|&r| r.to_be_bytes())
            .flatten()
            .collect::<Vec<u8>>()
            .try_into()
            .expect("input slice must contain exactly 2 u16 values");

        u32::from_be_bytes(bytes)
    }
}
