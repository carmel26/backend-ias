FROM rust:1.98-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY src ./src

ENV CARGO_BUILD_JOBS=1
ENV CARGO_INCREMENTAL=0

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/backend /app/backend

ENV SERVER_HOST=0.0.0.0

EXPOSE 3000

CMD ["/app/backend"]