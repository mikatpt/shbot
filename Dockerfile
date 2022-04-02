# syntax=docker/dockerfile:1
FROM rust:1.59.0

###### Caching build dependencies
# https://blog.mgattozzi.dev/caching-rust-docker-builds/

COPY _dummy.rs ./

COPY Cargo.lock ./
COPY Cargo.toml ./

RUN sed -i 's/src\/main.rs/_dummy.rs/' Cargo.toml

RUN cargo build --release

RUN sed -i 's/_dummy.rs/src\/main.rs/' Cargo.toml

######

COPY ./src ./src

# Production env variables
COPY ./.env.prod ./.env

RUN cargo build --release
CMD ["./target/release/shbot"]
