use chrono::{DateTime, LocalResult, TimeZone, Utc};
use rdkafka::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Created(pub DateTime<Utc>);

impl Default for Created {
    fn default() -> Self {
        Self(Utc::now())
    }
}

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
