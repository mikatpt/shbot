# syntax=docker/dockerfile:1
FROM rust:1.59.0

COPY ./ ./

RUN cp ./.env.prod ./.env

RUN cargo build --release
CMD ["./target/release/shereebot"]
