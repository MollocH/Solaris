use std::ops::Div;
use ascii_converter::*;
use hex;

pub trait Convert {
    fn convert_to_string(&self) -> String;
    fn convert_to_hex(&self) -> String;
    fn convert_to_decimal10(&self) -> f64;
}

impl Convert for Vec<u16> {
    fn convert_to_string(&self) -> String {
        let result = convert_vec_u16_to_vec_u8(&self);
        return decimals_to_string(&result).unwrap();
    }

    fn convert_to_hex(&self) -> String {
        let result = convert_vec_u16_to_vec_u8(&self);
        return hex::encode(&result);
    }

    fn convert_to_decimal10(&self) -> f64 {
        return f64::from(self[0]).div(f64::from(10.0));
    }
}

fn convert_vec_u16_to_vec_u8(data: &Vec<u16>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    for register in data {
        let mut bytes: Vec<u8> = register.to_be_bytes().to_vec();
        bytes.retain(|value: &u8| -> bool { *value != 0 });
        result.append(&mut bytes)
    }

    return result;
}
