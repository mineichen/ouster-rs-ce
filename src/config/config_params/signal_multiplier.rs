use serde::{Deserialize, Serialize};

use crate::InvalidConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SignalMultiplier {
    Quarter,
    Half,
    #[default]
    One,
    Two,
    Three,
}

impl SignalMultiplier {
    pub fn maximum_supported_azimuth_angle_deg(&self) -> u32 {
        match self {
            SignalMultiplier::Quarter | SignalMultiplier::Half | SignalMultiplier::One => 360,
            SignalMultiplier::Two => 180,
            SignalMultiplier::Three => 120,
        }
    }
}

impl Serialize for SignalMultiplier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SignalMultiplier::Quarter => serializer.serialize_f32(0.25),
            SignalMultiplier::Half => serializer.serialize_f32(0.5),
            SignalMultiplier::One => serializer.serialize_f32(1.),
            SignalMultiplier::Two => serializer.serialize_f32(2.),
            SignalMultiplier::Three => serializer.serialize_f32(3.),
        }
    }
}

impl<'de> Deserialize<'de> for SignalMultiplier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = f32::deserialize(deserializer)?;
        raw.try_into()
            .map_err(<D::Error as serde::de::Error>::custom)
    }
}

impl TryFrom<f32> for SignalMultiplier {
    type Error = InvalidConfig;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value == 0.25 {
            Ok(Self::Quarter)
        } else if value == 0.5 {
            Ok(Self::Half)
        } else if value == 1. {
            Ok(Self::One)
        } else if value == 2. {
            Ok(Self::Two)
        } else if value == 3. {
            Ok(Self::Three)
        } else {
            Err(InvalidConfig {
                reason: format!("Invalid signal multiplier: {value}"),
            })
        }
    }
}
