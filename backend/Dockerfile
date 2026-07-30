# --- build stage ---
FROM rust:1.80-slim AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

# --- runtime stage ---
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/homelab-sentinel /usr/local/bin/homelab-sentinel
EXPOSE 8087
ENTRYPOINT ["/usr/local/bin/homelab-sentinel"]
