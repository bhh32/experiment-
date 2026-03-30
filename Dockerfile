FROM ubuntu:24.04 AS builder

RUN apt-get update && apt-get install -y \
    curl build-essential cmake pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup target add wasm32-unknown-unknown
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall dioxus-cli@0.7.4 --no-confirm

WORKDIR /app
COPY . .

# Build backend
RUN cargo build --release --package backend

# Build frontend WASM
RUN cd crates/frontend && dx build --release

# Copy dx output to a known location
RUN cp -r /app/target/dx/frontend/release/web/public /app/frontend-dist

FROM ubuntu:24.04

RUN apt-get update && apt-get install -y pandoc && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/frontend-dist /app/public

# Copy custom assets
COPY crates/frontend/assets/ /app/public/assets/

ENV DATA_DIR=/data
ENV PUBLIC_DIR=/app/public

EXPOSE 8080

CMD ["/app/backend"]
