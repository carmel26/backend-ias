FROM rust:1.88-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN cargo install cargo-chef

RUN cargo chef prepare --recipe-path recipe.json

RUN CARGO_BUILD_JOBS=1 cargo chef cook --release --recipe-path recipe.json

COPY src ./src

RUN CARGO_BUILD_JOBS=1 cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/backend /app/backend

ENV SERVER_HOST=0.0.0.0

EXPOSE 3000

CMD ["/app/backend"]