use std::{cmp, fmt, fs::File, io::{BufReader, BufWriter, Write}, marker::PhantomData};

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
    latitude_e7: i32,
    #[serde(rename(deserialize = "longitudeE7"))]
    longitude_e7: i32,
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
            let mut cur_year = "2013".to_string();
            let mut writer = BufWriter::new(File::create("2013.tsv").unwrap());

            let mut result = Vec::new();
            while let Some(l) = seq.next_element::<Location>()? {
                let year = &l.timestamp[..4];
                if year != "2013" && year != "2014" {
                    continue;
                }
                if year != cur_year {
                    let new_path = format!("{}.tsv", year);
                    writer = BufWriter::new(File::create(new_path).unwrap());
                    cur_year = year.to_string();
                }

                writeln!(&mut writer, "{}\t{},{}", l.timestamp, l.longitude_e7 as f64 / 10000000.0, l.latitude_e7 as f64 / 10000000.0).unwrap();

                if result.len() >= 1000 {
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
