use chrono::{DateTime, Local, TimeZone, offset::LocalResult};
use color_eyre::{Result, eyre::bail};
use matrix_sdk::ruma::MilliSecondsSinceUnixEpoch;

pub trait ChronoExt {
    fn origin_server_chrono(&self) -> Result<DateTime<Local>>;
}

impl ChronoExt for MilliSecondsSinceUnixEpoch {
    fn origin_server_chrono(&self) -> Result<DateTime<Local>> {
        let timestamp_seconds: i64 = self.as_secs().into();

        let datetime = match Local.timestamp_opt(timestamp_seconds, 0) {
            LocalResult::Single(datetime) => datetime,
            LocalResult::Ambiguous(earlier_datetime, _later_datetime) => earlier_datetime,
            LocalResult::None => {
                bail!(
                    "Failed to convert message sent time of {} to local timestamp",
                    timestamp_seconds
                );
            }
        };

        Ok(datetime)
    }
}
