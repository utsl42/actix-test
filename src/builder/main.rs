extern crate mtbl;
extern crate serde_cbor;
extern crate serde_json;
use std::io;
use std::fs::File;
use mtbl::{Sorter, Write};

// Change the alias to `Box<error::Error>`.
type Result<T> = std::result::Result<T, Box<std::error::Error>>;

fn main() {
    read_file().unwrap();
}

fn read_file() -> Result<()> {
    let br = io::BufReader::new(File::open("countries.json")?);
    let data: serde_json::Value = serde_json::from_reader(br)?;

    let mut writer =
        Sorter::create_from_path("countries.mtbl", mtbl::Merger::merge_choose_first_value)?;
    if data.is_array() {
        let decoded: &Vec<serde_json::Value> = data.as_array().unwrap();
        for object in decoded.iter() {
            if let Some(&serde_json::Value::String(ref name)) = object.pointer("/name/common") {
                let _ = writer.add(name, serde_cbor::to_vec(object)?);
            }
            if let Some(&serde_json::Value::String(ref name)) = object.pointer("/cca3") {
                let _ = writer.add(name, serde_cbor::to_vec(object)?);
            }
        }
    }
    Ok(())
}
