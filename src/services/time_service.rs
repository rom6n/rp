use chrono::{DateTime, TimeZone, Duration, Utc};
use crate::models::TimeCustom;
use log::error;

impl TimeCustom {
    pub async fn from_usize_to_timestampz(time: usize) -> Result<String, String> {
        let time2: i64 = match time.try_into() {
            Ok(val) => val,
            Err(e) => {
                error!("Не удалось превратить usize в i64: {e}");
                return Err("TryFromIntError".to_string());
            }
        };

        let time3 = match Utc.timestamp_opt(time2, 0).single() {
            Some(val) => val,
            None => {
                error!("Не удалось поставить timestamp для i64: None");
                return Err("None".to_string())
            }
        };

        Ok(time3.to_rfc3339())
    }
}