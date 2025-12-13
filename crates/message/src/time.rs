use chrono::{DateTime, LocalResult, TimeZone, Utc};
#[cfg(feature = "iggy")]
use iggy::prelude::IggyTimestamp;
#[cfg(feature = "kafka")]
use rdkafka::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Created(pub DateTime<Utc>);

impl Default for Created {
    fn default() -> Self {
        Self(Utc::now())
    }
}

impl From<u64> for Created {
    fn from(value: u64) -> Self {
        if let LocalResult::Single(ts) = Utc.timestamp_micros(value as i64) {
            return Self(ts);
        }
        match Utc.timestamp_millis_opt(0) {
            LocalResult::Single(ts) => Self(ts),
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "kafka")]
impl From<Timestamp> for Created {
    fn from(value: Timestamp) -> Self {
        if let Timestamp::CreateTime(ts) = value
            && let LocalResult::Single(ts) = Utc.timestamp_millis_opt(ts)
        {
            return Self(ts);
        }
        match Utc.timestamp_millis_opt(0) {
            LocalResult::Single(ts) => Self(ts),
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "iggy")]
impl From<IggyTimestamp> for Created {
    fn from(value: IggyTimestamp) -> Self {
        if let LocalResult::Single(ts) = Utc.timestamp_micros(value.as_micros() as i64) {
            return Self(ts);
        }
        match Utc.timestamp_millis_opt(0) {
            LocalResult::Single(ts) => Self(ts),
            _ => unreachable!(),
        }
    }
}
