FROM rust:1.47 as builder
WORKDIR /usr/src/gitrello-github-integration-service
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install ca-certificates libssl-dev libpq-dev -y && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/gitrello-github-integration-service /usr/local/bin/gitrello-github-integration-service
CMD ["gitrello-github-integration-service"]
