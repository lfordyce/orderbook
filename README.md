# Order Book processing

## Summary

This program reads a series of new orders, order cancellations, and flushes from a CSV, processes each order transaction,
and outputting the result to stdout as a CSV. Errors and additional information are logged to stderr.

## Quick Start

- Build and run the unit test

```shell
cargo test
```

## Usage
```shell
cargo run -- --help
```

```shell
Usage: orderbook [OPTIONS]

Options:
  -i, --input <ORDER FILE SOURCE>
  -h, --help                       Print help
```

## Run options

```shell
# Print results to standard out
cat etc/input_file.csv | cargo run
# Capture results into .csv file
cat etc/input_file.csv | cargo run > results.csv
# Alternatively with `Release` optimization
cat etc/input_file.csv | cargo run --release
# Alternatively supply input file path rather than reading from stdin
cargo run --release -- --input=etc/input_file.csv
```

## Docker option
- Build image
```shell
docker build -t orderbook .
```
- Run
```shell
cat etc/input_file.csv | docker run -i orderbook
```

## Error Handling

- Fatal Errors i.e. IO errors are logged to stderr.

