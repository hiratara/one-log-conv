## Converting Google Location History JSON into KML supported by 1log

`one-log-conv` converts Google Location History of [Google Takeout](https://takeout.google.com/settings/takeout) into KML format that you can import into [1log](https://1log.app/).

## How to use

1. Download your Google Location History from the Google Takeout page

2. Extract `Records.json` from it

You can use the `unzip` command to do so.

```
$ unzip -p /path/to/takeout-YOUR.zip 'Takeout/Location History/Records.json' > Records.json
```

3. Build `one-log-conv` with `--release` flag

```
$ cargo build --release
```

4. make the `output` directory

```
$ mkdir output/
```

5. run the command

```
$ ./target/release/one-log-conv
```

It creates KML files for each year and month, like `output/2023-03.kml`.

6. import `output/*.kml` by using the 1log app
