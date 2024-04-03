use std::{borrow::Cow, fmt::Debug, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
#[non_exhaustive]
pub enum LidarProfile {
    #[default]
    SingleReturn,
    DualReturn,
    LowData,
}

impl Serialize for LidarProfile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            LidarProfile::SingleReturn => "RNG19_RFL8_SIG16_NIR16",
            LidarProfile::DualReturn => "RNG19_RFL8_SIG16_NIR16_DUAL",
            LidarProfile::LowData => "RNG15_RFL8_NIR8",
        })
    }
}

impl<'de> Deserialize<'de> for LidarProfile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let x = Cow::<str>::deserialize(deserializer)?;
        Self::from_str(&x).map_err(<D::Error as serde::de::Error>::custom)
    }
}

impl FromStr for LidarProfile {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RNG19_RFL8_SIG16_NIR16" => Ok(Self::SingleReturn),
            "RNG15_RFL8_NIR8" => Ok(Self::LowData),
            "RNG19_RFL8_SIG16_NIR16_DUAL" => Ok(Self::DualReturn),
            s => Err(format!("Can't parse '{}' into LidarProfile", s).into()),
        }
    }
}
