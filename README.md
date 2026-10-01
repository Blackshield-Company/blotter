# blotter

**blotter** normalizes and explores public crime/incident data. Every city publishes incident data as a differently-shaped CSV — different column names, different date formats, different offense labels. blotter applies a small per-city TOML mapping config to turn any of them into **one common schema**, so journalists and researchers can compare cities without writing a new scraper script each time.

It is fully **local-first and offline**: blotter never touches the network. Downloading your city's open-data CSV is your job; everything after that happens on your machine.

## Who it's for

- **Journalists** comparing incident trends across cities with incompatible data portals.
- **Researchers** who need a clean, uniform CSV (or JSON) to feed into their analysis tools.
- **Civic hackers** building on top of municipal open data.

## Install

```sh
cargo install --path .
```

Requires a stable Rust toolchain (edition 2021).

## Quickstart

The repo ships a fully synthetic example: a raw CSV from the fictional **City of Exampletown** and a matching mapping config. (All street names, places, and URLs are invented — nothing real.)

```sh
# 1. Normalize the raw Exampletown export into the common schema
blotter normalize \
  --config examples/exampletown-mapping.toml \
  --input  examples/exampletown-incidents.csv \
  --output normalized.csv

# 2. Explore it
blotter stats normalized.csv
blotter to-json normalized.csv
```

## The common schema

Every normalized CSV has exactly these columns:

| column | required | notes |
|---|---|---|
| `incident_id` | yes | source's case/report number |
| `datetime` | yes | ISO-style `YYYY-MM-DDTHH:MM:SS`, parsed via the config's format string |
| `category` | yes | one of the standard categories below |
| `description` | yes | free text (empty if the source has none) |
| `latitude` | optional | decimal degrees |
| `longitude` | optional | decimal degrees |
| `district_or_neighborhood` | optional | beat, precinct, neighborhood — whatever the city calls it |
| `source_city` | yes | stamped from the mapping config |
| `source_url` | optional | link back to the source record |

**Standard categories:** `theft`, `assault`, `burglary`, `robbery`, `vandalism`, `fraud`, `homicide`, `drug`, `traffic`, `other`. Any source category you don't explicitly map falls back to `other`.

## Writing a mapping config for a new city

Download the city's incident CSV, look at its header row, and write a TOML file with four parts:

```toml
# 1. City name stamped onto every output row
source_city = "Exampletown"

# 2. chrono-style format string matching the source's datetime column.
#    Date-only formats work too (rows become midnight).
datetime_format = "%m/%d/%Y %H:%M"

# 3. Column mapping: schema field = source column name.
#    Optional fields (description, latitude, longitude,
#    district_or_neighborhood, source_url) can simply be omitted.
[columns]
incident_id = "CASE_NUM"
datetime = "OCCURRED"
category = "OFFENSE"
description = "SUMMARY"
latitude = "LAT"
longitude = "LON"
district_or_neighborhood = "BEAT_NAME"
source_url = "RECORD_URL"

# 4. Category remap: raw source strings -> standard categories.
#    Matching is case-insensitive; unmapped values become "other".
[categories]
"LARCENY / THEFT" = "theft"
"AGGRAVATED ASSAULT" = "assault"
"BURGLARY - RESIDENTIAL" = "burglary"
"ARMED ROBBERY" = "robbery"
"NARCOTICS POSSESSION" = "drug"
"TRAFFIC COLLISION - DUI" = "traffic"
```

Tips:

- Run `blotter stats` on your output and check the `other` bucket — anything large in there means your `[categories]` table is missing source values.
- Rows with unparseable datetimes or missing IDs are skipped with a warning, not fatal.
- The full set of chrono format tokens: <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>

## Commands

```
blotter normalize --config MAPPING.toml --input IN.csv --output OUT.csv
blotter stats NORMALIZED.csv        # counts by category, month, district
blotter to-json NORMALIZED.csv      # pretty-printed JSON array on stdout
```

## Development

```sh
cargo test    # unit + end-to-end tests against the bundled synthetic example
```

## Roadmap

- Mapping configs for real cities, contributed by users
- `blotter diff` — compare two normalized datasets
- Dashboard UI for interactive exploration (later)
- SQLite export for larger datasets

## License

Apache-2.0 — see [LICENSE](LICENSE). Example data in `examples/` is entirely synthetic.


Part of [Blackshield Company](https://github.com/Blackshield-Company).
