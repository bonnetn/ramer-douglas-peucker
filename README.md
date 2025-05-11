# trajectory-rs

A proof-of-concept Rust tool for processing, simplifying, and serializing GPS trajectory data (e.g., from the [Geolife](https://www.microsoft.com/en-us/research/publication/geolife-gps-trajectory-dataset-user-guide/) dataset).

## Features
- Reads Geolife-format `.plt` files
- Sorts and processes GPS points
- Simplifies trajectories using the Douglas-Peucker algorithm
- Serializes to Protocol Buffers (with and without delta encoding)
- Prints statistics about compression and simplification

## Usage

1. **Fetch the Geolife data** (see `fetch_data.sh`)
2. **Build and run:**
   ```sh
   cargo run --release
   ```
   By default, reads from the `geolife/` directory.
