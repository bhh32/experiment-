FROM rust:1.86 AS builder

RUN rustup target add wasm32-unknown-unknown
RUN cargo install dioxus-cli --version 0.7.4

WORKDIR /app
COPY . .

# Build backend
RUN cargo build --release --package backend

# Build frontend WASM
RUN cd crates/frontend && dx build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y pandoc && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/crates/frontend/target/dx/docs-clone/release/web/public /app/public

# Copy custom assets
COPY crates/frontend/assets/ /app/public/assets/

ENV DATA_DIR=/data
ENV PUBLIC_DIR=/app/public

EXPOSE 8080

CMD ["/app/backend"]
