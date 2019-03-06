use sled;
use serde_cbor;
use serde_json;
use std::fs::File;
use std::io;

// Change the alias to `Box<error::Error>`.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let br = io::BufReader::new(File::open("countries.json")?);
    let data: serde_json::Value = serde_json::from_reader(br)?;

    let db = sled::Db::start_default("countries_db")?.open_tree(b"countries".to_vec())?;
    if data.is_array() {
        let decoded: &Vec<serde_json::Value> = data.as_array().unwrap();
        for object in decoded.iter() {
            if let Some(&serde_json::Value::String(ref name)) = object.pointer("/name/common") {
                db.set(name, serde_cbor::to_vec(object)?)?;
            }
            if let Some(&serde_json::Value::String(ref name)) = object.pointer("/cca3") {
                db.set(name, serde_cbor::to_vec(object)?)?;
            }
        }
    }
    db.flush()?;
    Ok(())
}
