use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

use crate::{InvalidConfig, LidarProfile};

mod azimuth_window;
mod signal_multiplier;

pub use azimuth_window::*;
pub use signal_multiplier::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigParamsRaw {
    pub azimuth_window: AzimuthWindow,
    pub lidar_mode: LidarMode,
    pub udp_dest: Ipv4Addr,
    pub udp_port_lidar: u16,
    pub udp_profile_lidar: LidarProfile,
    pub signal_multiplier: SignalMultiplier,
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

impl From<ConfigParams> for ConfigParamsRaw {
    fn from(value: ConfigParams) -> Self {
        value.0
    }
}

impl TryFrom<ConfigParamsRaw> for ConfigParams {
    type Error = InvalidConfig;

    fn try_from(value: ConfigParamsRaw) -> Result<Self, Self::Error> {
        let allowed_deg = value
            .signal_multiplier
            .maximum_supported_azimuth_angle_deg();
        let given_milli_deg = value.azimuth_window.milli_angle_deg();
        if allowed_deg * 1000 < given_milli_deg {
            Err(InvalidConfig {
                reason: format!(
                    "Azimuth-Angle is too big for signal_multiplier({:?}): Allowed({} deg) > Window({} millideg)",
                    value.signal_multiplier,
                    allowed_deg,
                    given_milli_deg
                ),
            })
        } else {
            Ok(ConfigParams(value))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LidarMode {
    #[serde(rename = "512x10")]
    Mode512x10,
    #[serde(rename = "512x20")]
    Mode512x20,
    #[default]
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
