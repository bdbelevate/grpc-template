use serde::Deserialize;
use std::fs::File;
use std::time::SystemTime;

use crate::{{crate_name}}::Timestamp;

#[derive(Debug, Deserialize)]
struct Sample {
    id: String,
    name: String,
    description: Option<String>,
    created_at: Option<i64>,
}

fn now() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

#[allow(dead_code)]
pub fn load_items() -> Vec<crate::{{crate_name}}::Sample> {
    let file = File::open("examples/data/samples.json").expect("failed to open data file");

    let decoded: Vec<Sample> =
        serde_json::from_reader(&file).expect("failed to deserialize features");

    decoded
        .into_iter()
        .map(|item| crate::{{crate_name}}::Sample {
            id: "None".to_string(),
            name: item.name,
            description: item.description.unwrap_or("".to_string()),
            created_at: Some(Timestamp {
                seconds: item.created_at.unwrap_or(now() as i64),
                nanos: 0,
            }),
        })
        .collect()
}
