use std::time::{SystemTime, SystemTimeError};
use rkyv::{Archive, Deserialize, Serialize};

/// A wrapper for a signed 64-bit integer representing milliseconds
/// from the Unix epoch.
#[derive(Archive, Serialize, Deserialize, PartialOrd, PartialEq, Ord, Eq)]
pub(crate) struct DateTime {
    timestamp: i64
}

impl From<i64> for DateTime {
    fn from(timestamp: i64) -> Self {
        Self {
            timestamp
        }
    }
}

impl TryFrom<SystemTime> for DateTime {
    type Error = SystemTimeError;

    fn try_from(value: SystemTime) -> Result<Self, Self::Error> {
        let since_epoch = value.duration_since(SystemTime::UNIX_EPOCH)?;
        let date_time = Self {
            timestamp: since_epoch.as_millis() as i64
        };

        Ok(date_time)
    }
}