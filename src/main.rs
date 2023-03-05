use std::{fs::File, io::BufReader};

fn main() {
    let file = File::open("Records.json").unwrap();
    let file = BufReader::new(file);
    let v: serde_json::Value = serde_json::from_reader(file).unwrap();
    match v {
        serde_json::Value::Object(obj) => println!("{:?}", obj.keys().collect::<Vec<_>>()),
        _ => println!("sorry"),
    }
}

// TODO: without buffering https://serde.rs/stream-array.html