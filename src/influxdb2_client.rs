use async_trait::async_trait;
use futures::prelude::*;
use influxdb2::Client;
use influxdb2::models::DataPoint;

use super::app_config::AppConfig;
use super::app_config::Influxdb2;

pub struct SolarisInfluxdb2Client {
    client: Client,
    bucket: String
}

#[async_trait]
pub trait Influxdb2Client {
    fn new(influxdb2_config: &Influxdb2) -> Self;
    async fn write(&self, datapoints: Vec<DataPoint>);
}

#[async_trait]
impl Influxdb2Client for SolarisInfluxdb2Client {
    fn new(influxdb2_config: &Influxdb2) -> Self {
        let client = Client::new(&influxdb2_config.uri, &influxdb2_config.org, &influxdb2_config.token);
        return SolarisInfluxdb2Client {
            client,
            bucket: influxdb2_config.bucket.clone()
        }
    }

    async fn write(&self, datapoints: Vec<DataPoint>) {
        let result = self.client.write(&self.bucket, stream::iter(datapoints)).await;
        result.unwrap();
    }
}
