# Order Book processing

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