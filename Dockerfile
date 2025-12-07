FROM rust:1.91 as builder

WORKDIR /app

# Pre-copy manifests to leverage Docker layer caching for dependencies.
COPY Cargo.toml Cargo.lock .env ./
COPY backend backend

RUN cargo build --locked --release --bin local-guide-backend

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Where uploaded place images are stored.
ENV PLACE_IMAGE_DIR=/app/data/place_images
RUN mkdir -p "${PLACE_IMAGE_DIR}"

COPY --from=builder /app/target/release/local-guide-backend /usr/local/bin/local-guide-backend

EXPOSE 8080

CMD ["local-guide-backend"]
