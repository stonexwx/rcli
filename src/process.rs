use core::str;
use csv::Reader;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    city: String,
    region: String,
    country: String,
    population: String,
}

pub fn process_csv(input: &str, output: &str) -> anyhow::Result<()> {
    let mut reader = Reader::from_path(input)?;
    let mut ret = Vec::with_capacity(128);
    for result in reader.deserialize() {
        let record: Record = result?;
        ret.push(record);
    }
    let json = serde_json::to_string(&ret)? + "\n";
    fs::write(output, json)?;
    Ok(())
}
