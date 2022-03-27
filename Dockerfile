# syntax=docker/dockerfile:1
FROM rust:1.59.0

COPY ./ ./

RUN cargo build --release
CMD ["./target/release/shereebot"]
