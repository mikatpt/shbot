# syntax=docker/dockerfile:1

## https://www.lpalmieri.com/posts/fast-rust-docker-builds/

# 1. Using cargo-chef, compute the recipe needed to build dependencies.

FROM rust:1.59.0 AS chef
RUN cargo install cargo-chef

WORKDIR shbot

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# 2. Build dependencies

FROM chef AS builder 

COPY --from=planner /shbot/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# 3. Build application

COPY . .

RUN cargo build --release --bin shbot

# 3. Run app in a default ubuntu container. We don't need rust, since we just
#    copy the binary over from `builder`

FROM rust:1.59.0 AS runtime
RUN apt-get update && apt-get install -y libssl-dev
WORKDIR shbot

## Prod env variables
ARG SHBOT_ENV_FILE

COPY ${SHBOT_ENV_FILE} .env
COPY --from=builder /shbot/target/release/shbot /usr/local/bin
ENTRYPOINT ["/usr/local/bin/shbot"]
