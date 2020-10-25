FROM rust:1.47 as builder
RUN USER=root cargo new gitrello-github-integration-service --bin --vcs none
WORKDIR ./gitrello-github-integration-service

COPY Cargo.lock Cargo.toml ./
RUN cargo build --release
RUN rm src/*.rs

COPY src src
COPY diesel.toml diesel.toml
RUN rm ./target/release/deps/gitrello_github_integration_service*
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get install ca-certificates libssl-dev libpq-dev -y && rm -rf /var/lib/apt/lists/*
COPY --from=builder /gitrello-github-integration-service/target/release/gitrello-github-integration-service /usr/local/bin/gitrello-github-integration-service
CMD ["gitrello-github-integration-service"]
