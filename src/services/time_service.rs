use chrono::{DateTime, TimeZone, Utc};
use crate::models::{TimeCustom, TimeCustomError};
use log::error;

impl TimeCustom {
    pub async fn from_usize_to_timestampz(time: usize) -> Result<DateTime<Utc>, TimeCustomError> {
        let time2: i64 = match time.try_into() {
            Ok(val) => val,
            Err(e) => {
                error!("Не удалось превратить usize в i64: {e}");
                return Err(TimeCustomError::ParseError);
            }
        };

        let time3 = match Utc.timestamp_opt(time2, 0).single() {
            Some(val) => val,
            None => {
                error!("Не удалось поставить timestamp для i64: None");
                return Err(TimeCustomError::TimestampError)
            }
        };

        Ok(time3)
    }
}