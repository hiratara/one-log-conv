use std::{
    cmp,
    collections::HashSet,
    fmt,
    fs::File,
    io::{BufReader, BufWriter, Write},
    marker::PhantomData,
};

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

    impl<'de> Visitor<'de> for LocVisitor {
        type Value = Vec<Location>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a nonempty sequence of numbers")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut done = HashSet::new();
            let mut current: Option<(String, BufWriter<File>)> = None;
            // let mut cur_year = "2013".to_string();
            // let mut writer = new_file(&cur_year);

            let mut result = Vec::new();
            while let Some(l) = seq.next_element::<Location>()? {
                let year_month = &l.timestamp[..7];
                if !year_month.starts_with("2013") && !year_month.starts_with("2014") {
                    continue;
                }

                current = match current {
                    None => {
                        let writer = new_file(&year_month, &mut done);
                        Some((year_month.to_owned(), writer))
                    }
                    Some((cur_year_month, cur_writer)) => {
                        if year_month == cur_year_month {
                            Some((cur_year_month, cur_writer))
                        } else {
                            end_file(cur_writer);
                            let writer = new_file(&year_month, &mut done);
                            Some((year_month.to_owned(), writer))
                        }
                    }
                };

                match current {
                    None => panic!("current file not found: {}", year_month),
                    Some((_, ref mut writer)) => {
                        write!(writer, r#"<Placemark>"#).unwrap();
                        write!(
                            writer,
                            r#"<TimeStamp><when>{}</when></TimeStamp>"#,
                            l.timestamp
                        )
                        .unwrap();
                        write!(
                            writer,
                            r#"<Point><coordinates>{},{}</coordinates></Point>"#,
                            l.longitude_e7 as f64 / 10000000.0,
                            l.latitude_e7 as f64 / 10000000.0
                        )
                        .unwrap();
                        write!(writer, r#"<ExtendedData><Data name="activeFlag"><value>true</value></Data></ExtendedData></Placemark>"#).unwrap();
                        write!(writer, "\n").unwrap();
                        // writeln!(&mut writer, "{}\t{},{}", l.timestamp, l.longitude_e7 as f64 / 10000000.0, l.latitude_e7 as f64 / 10000000.0).unwrap();
                    }
                };

                if result.len() >= 1000 {
                    continue;
                }
                result.push(l);
            }

            match current {
                None => println!("no locations found"),
                Some((_, writer)) => end_file(writer),
            };

            Ok(result)
        }
    }

    // Create the visitor and ask the deserializer to drive it. The
    // deserializer will call visitor.visit_seq() if a seq is present in
    // the input data.
    let visitor = LocVisitor;
    deserializer.deserialize_seq(visitor)
}

fn new_file(year_month: &str, done: &mut HashSet<String>) -> BufWriter<File> {
    if done.contains(year_month) {
        panic!("Has already opened {}", year_month);
    }
    done.insert(year_month.to_owned());

    let path = format!("output/{}.kml", year_month);
    let file = File::create(path).unwrap();
    let mut writer = BufWriter::new(file);

    write!(&mut writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#).unwrap();
    write!(
        &mut writer,
        r#"<kml xmlns="http://earth.google.com/kml/2.2">"#
    )
    .unwrap();
    write!(&mut writer, r#"<Document>"#).unwrap();
    write!(&mut writer, r#"<name>1log location logs</name>"#).unwrap();
    write!(writer, "\n").unwrap();

    writer
}

fn end_file(mut writer: BufWriter<File>) {
    write!(&mut writer, r#"</Document>"#).unwrap();
    write!(&mut writer, r#"</kml>"#).unwrap();
    write!(writer, "\n").unwrap();
}
