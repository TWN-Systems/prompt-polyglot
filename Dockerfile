# syntax=docker/dockerfile:1.6

FROM rust:1.75-bullseye AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

COPY . .

RUN cargo build --release --bin prompt-compress-server --bin prompt-compress-healthcheck

FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app

COPY --from=builder /app/target/release/prompt-compress-server /usr/local/bin/prompt-compress-server
COPY --from=builder /app/target/release/prompt-compress-healthcheck /usr/local/bin/prompt-compress-healthcheck
COPY --from=builder /app/data ./data

EXPOSE 8080
ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/prompt-compress-server"]
