use core::str;
use csv::Reader;
use serde_json::Value;
use std::fs;

pub fn process_csv(input: &str, output: &str) -> anyhow::Result<()> {
    let mut reader = Reader::from_path(input)?;
    let mut ret = Vec::with_capacity(128);
    let headers = reader.headers()?.clone();
    for result in reader.records() {
        let record = result?;
        let json_value = headers.iter().zip(record.iter()).collect::<Value>();
        ret.push(json_value);
    }

    let json = serde_json::to_string(&ret)?;
    fs::write(output, json)?;
    Ok(())
}
