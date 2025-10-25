# syntax=docker/dockerfile:1.6

FROM rust:1.75-bullseye AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN cargo fetch --locked

COPY . .

RUN cargo build --locked --bin prompt-compress-server --release

FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app

COPY --from=builder /app/target/release/prompt-compress-server /usr/local/bin/prompt-polyglot
COPY --from=builder /app/data ./data

ENV RUST_LOG=info
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/prompt-polyglot"]
