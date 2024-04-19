use core::str;
use csv::Reader;
use serde_json::Value;
use std::fs;

use crate::opts::OutputFormat;

pub fn process_csv(input: &str, output: String, output_format: OutputFormat) -> anyhow::Result<()> {
    let mut reader = Reader::from_path(input)?;
    let mut ret = Vec::with_capacity(128);
    let headers = reader.headers()?.clone();
    for result in reader.records() {
        let record = result?;
        let json_value = headers.iter().zip(record.iter()).collect::<Value>();
        ret.push(json_value);
    }
    let ret = match output_format {
        OutputFormat::Json => serde_json::to_string(&ret)?,
        OutputFormat::Yaml => serde_yaml::to_string(&ret)?,
        _ => anyhow::bail!("Unsupporteded output format"),
    };
    fs::write(output, ret)?;
    Ok(())
}
