use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigParamsRaw {
    pub azimuth_window: [u32; 2],
    pub lidar_mode: LidarMode,
    pub udp_dest: Ipv4Addr,
    pub udp_port_lidar: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidateConfigParamsError {
    #[error("Invalid range")]
    InvalidRange,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ConfigParams(ConfigParamsRaw);

impl std::ops::Deref for ConfigParams {
    type Target = ConfigParamsRaw;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ConfigParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = ConfigParamsRaw::deserialize(deserializer)?;
        inner
            .try_into()
            .map_err(<D::Error as serde::de::Error>::custom)
    }
}

impl TryFrom<ConfigParamsRaw> for ConfigParams {
    type Error = ValidateConfigParamsError;

    fn try_from(value: ConfigParamsRaw) -> Result<Self, Self::Error> {
        if value.azimuth_window[0] > 360_000 || value.azimuth_window[1] > 360_000 {
            Err(ValidateConfigParamsError::InvalidRange)
        } else {
            Ok(ConfigParams(value))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LidarMode {
    #[serde(rename = "512x10")]
    Mode512x10,
    #[serde(rename = "512x20")]
    Mode512x20,
    #[serde(rename = "1024x10")]
    Mode1024x10,
    #[serde(rename = "1024x20")]
    Mode1024x20,
    #[serde(rename = "2048x10")]
    Mode2048x10,
}

impl LidarMode {
    pub fn horizontal_resolution(&self) -> u16 {
        match self {
            LidarMode::Mode512x10 | LidarMode::Mode512x20 => 512,
            LidarMode::Mode1024x10 | LidarMode::Mode1024x20 => 1024,
            LidarMode::Mode2048x10 => 2048,
        }
    }
}
