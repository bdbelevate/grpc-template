use std::time::SystemTime;

pub mod error;
pub mod samples;

use crate::{{crate_name}}::Timestamp;

pub fn get_timestamp() -> Timestamp {
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    Timestamp {
        seconds: ts.as_secs() as i64,
        nanos: (ts.as_nanos() - ts.as_secs() as u128 * 1_000_000_000) as i32,
    }
}
