use hex;
use std::ops::Div;

pub trait Convert {
    fn convert_to_string(&self) -> String;
    fn convert_to_hex(&self) -> String;
    fn convert_to_decimal10(&self) -> f64;
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

    fn convert_to_decimal10(&self) -> f64 {
        f64::from(self[0]).div(f64::from(10.0))
    }
}
