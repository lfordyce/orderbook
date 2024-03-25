FROM rust:bookworm as builder

# Create a build directory and copy over all of the files
WORKDIR /build
COPY . ./

# Attempt to resuse a cache between build.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release && cp /build/target/release/orderbook /usr/local/cargo/bin

# orderbook image with just the binaries
FROM debian:bookworm-slim

COPY --from=builder /usr/local/cargo/bin/orderbook /usr/local/bin

ENTRYPOINT ["/usr/local/bin/orderbook"]