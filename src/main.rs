use std::{cmp, fmt, fs::File, io::BufReader, marker::PhantomData};

use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

fn main() {
    let file = File::open("Records.json").unwrap();
    let file = BufReader::new(file);
    let v: Records = serde_json::from_reader(file).unwrap();
    println!("{:?}", v);
}

// TODO: without buffering https://serde.rs/stream-array.html

#[derive(Debug, Deserialize)]
struct Records {
    #[serde(deserialize_with = "deserialize_max")]
    locations: Vec<Location>,
}

#[derive(Debug, Deserialize)]
struct Location {
    #[serde(rename(deserialize = "latitudeE7"))]
    latitude_e7: u32,
    #[serde(rename(deserialize = "longitudeE7"))]
    longitude_e7: u32,
    timestamp: String,
}

fn deserialize_max<'de, D>(deserializer: D) -> Result<Vec<Location>, D::Error>
where
    D: Deserializer<'de>,
{
    struct LocVisitor;

    impl<'de> Visitor<'de> for LocVisitor
    {
        type Value = Vec<Location>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a nonempty sequence of numbers")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut result = Vec::new();
            while let Some(l) = seq.next_element::<Location>()? {
                if result.len() >= 10 {
                    continue;
                }
                result.push(l);
            }

            Ok(result)
        }
    }

    // Create the visitor and ask the deserializer to drive it. The
    // deserializer will call visitor.visit_seq() if a seq is present in
    // the input data.
    let visitor = LocVisitor;
    deserializer.deserialize_seq(visitor)
}
