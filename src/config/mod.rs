use std::{fmt::Debug, ops::Deref};

use serde::Deserialize;

use crate::Profile;

mod beam_intrinsics;
mod config_params;
mod lidar_data_format;
mod lidar_profile;

pub use beam_intrinsics::*;
pub use config_params::*;
pub use lidar_data_format::*;
pub use lidar_profile::*;

/// Not Serializable, as it doesn't contain all values from the spec and won't be the same as when it's read again
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OusterConfig {
    pub beam_intrinsics: BeamIntrinsics,
    pub config_params: ConfigParams,
    pub lidar_data_format: LidarDataFormat,
}

/// Mustn't contain contradicting information like (window-size which doesnt't match Profile::Columns)
pub struct ValidOusterConfig<TProfile> {
    pub config_params: ConfigParams,
    pub valid_operation: ValidOperationConfig<TProfile>,
}

impl<TProfile> Deref for ValidOusterConfig<TProfile> {
    type Target = ValidOperationConfig<TProfile>;

    fn deref(&self) -> &Self::Target {
        &self.valid_operation
    }
}

pub struct ValidOperationConfig<TProfile> {
    pub beam_intrinsics: BeamIntrinsics,
    pub lidar_data_format: ValidLidarDataFormat<TProfile>,
}

impl<TProfile> ValidOperationConfig<TProfile> {
    pub fn n_vec(&self) -> u32 {
        self.beam_intrinsics.n_vec()
    }
}

impl<T: Profile> TryFrom<OusterConfig> for ValidOusterConfig<T> {
    type Error = InvalidConfig;

    fn try_from(value: OusterConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            config_params: value.config_params,
            valid_operation: ValidOperationConfig {
                beam_intrinsics: value.beam_intrinsics,
                lidar_data_format: value.lidar_data_format.try_into()?,
            },
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{reason}")]
pub struct InvalidConfig {
    reason: String,
}

impl InvalidConfig {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}
