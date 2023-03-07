use std::{
    collections::HashSet,
    fmt,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf, str::FromStr,
};

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};

fn main() {
    let file = File::open("Records.json").unwrap();
    let file = BufReader::new(file);
    let records: Records = serde_json::from_reader(file).unwrap();

    for r in records.conversion_results {
        println!("{}\t{}", r.path.display(), r.count);
    }
}

#[derive(Debug, Deserialize)]
struct Records {
    #[serde(deserialize_with = "deserialize_locations")]
    #[serde(rename(deserialize = "locations"))]
    conversion_results: Vec<ConversionResult>,
}

#[derive(Debug, Deserialize)]
struct Location {
    #[serde(rename(deserialize = "latitudeE7"))]
    latitude_e7: i32,
    #[serde(rename(deserialize = "longitudeE7"))]
    longitude_e7: i32,
    timestamp: String,
}

#[derive(Debug)]
struct ConversionResult {
    path: PathBuf,
    count: u64,
}

fn deserialize_locations<'de, D>(deserializer: D) -> Result<Vec<ConversionResult>, D::Error>
where
    D: Deserializer<'de>,
{
    struct LocVisitor;

    impl<'de> Visitor<'de> for LocVisitor {
        type Value = Vec<ConversionResult>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a nonempty sequence of numbers")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut results = Vec::new();

            let mut done = HashSet::new();
            let mut current: Option<(String, BufWriter<File>, ConversionResult)> = None;

            while let Some(l) = seq.next_element::<Location>()? {
                let year_month = &l.timestamp[..7];

                current = match current {
                    None => {
                        let (writer, result) = new_file(&year_month, &mut done);
                        Some((year_month.to_owned(), writer, result))
                    }
                    Some((cur_year_month, cur_writer, result)) => {
                        if year_month == cur_year_month {
                            Some((cur_year_month, cur_writer, result))
                        } else {
                            end_file(cur_writer, result, &mut results);
                            let (writer, result) = new_file(&year_month, &mut done);
                            Some((year_month.to_owned(), writer, result))
                        }
                    }
                };

                match current {
                    None => panic!("current file not found: {}", year_month),
                    Some((_, ref mut writer, ref mut result)) => {
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

                        result.count += 1;
                    }
                };
            }

            match current {
                None => println!("no locations found"),
                Some((_, writer, result)) => end_file(writer, result, &mut results),
            };

            Ok(results)
        }
    }

    // Create the visitor and ask the deserializer to drive it. The
    // deserializer will call visitor.visit_seq() if a seq is present in
    // the input data.
    let visitor = LocVisitor;
    deserializer.deserialize_seq(visitor)
}

fn new_file(year_month: &str, done: &mut HashSet<String>) -> (BufWriter<File>, ConversionResult) {
    if done.contains(year_month) {
        panic!("Has already opened {}", year_month);
    }
    done.insert(year_month.to_owned());

    let path = format!("output/{}.kml", year_month);
    let file = File::create(&path).unwrap();
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

    (writer, ConversionResult{path: PathBuf::from_str(&path).unwrap(), count:0})
}

fn end_file(mut writer: BufWriter<File>, result: ConversionResult, results: &mut Vec<ConversionResult>) {
    write!(&mut writer, r#"</Document>"#).unwrap();
    write!(&mut writer, r#"</kml>"#).unwrap();
    write!(writer, "\n").unwrap();

    results.push(result);
}
