# Order Book processing

## Summary

This program takes a CSV input of new orders, order cancellations, and flushes and processes each order transaction,
outputting the result to stdout. Errors and additional information is logged to stderr.

## Quick Start

- Build and run the unit test

```shell
cargo test
```

## Usage

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

## Error Handling

- Fatal Errors i.e. IO errors are logged to stderr.

