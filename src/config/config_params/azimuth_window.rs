use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct AzimuthWindow([u32; 2]);

impl TryFrom<[u32; 2]> for AzimuthWindow {
    type Error = &'static str;

    fn try_from(value: [u32; 2]) -> Result<Self, Self::Error> {
        if value[0] > 360_000 || value[1] > 360_000 {
            Err("Component musten't be > 360'000")
        } else {
            Ok(AzimuthWindow(value))
        }
    }
}

impl From<AzimuthWindow> for [u32; 2] {
    fn from(value: AzimuthWindow) -> Self {
        value.0
    }
}

impl AzimuthWindow {
    pub fn milli_angle_deg(&self) -> u32 {
        if self[0] < self[1] {
            self[1] - self[0]
        } else {
            360_000 - self[1] + self[0]
        }
    }
}

impl<'de> Deserialize<'de> for AzimuthWindow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = <[u32; 2]>::deserialize(deserializer)?;
        raw.try_into()
            .map_err(<D::Error as serde::de::Error>::custom)
    }
}

impl Deref for AzimuthWindow {
    type Target = [u32; 2];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
