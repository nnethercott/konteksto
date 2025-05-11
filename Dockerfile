FROM ghcr.io/astral-sh/uv:debian-slim AS scraper

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

WORKDIR /build

COPY konteksto-builder ./konteksto-builder
COPY scripts/build-db.sh .

RUN chmod +x build-db.sh
RUN ./build-db.sh

# build web and cli
FROM rust:slim-bookworm as app-builder
RUN apt-get update && apt-get install pkg-config libssl-dev -y
WORKDIR /app

# setup db
COPY migrations migrations
COPY .env .env
RUN cargo install sqlx-cli && \
  mkdir -p data/sqlite && \
  touch data/sqlite/app.db && \
  sqlx migrate run

COPY . .

RUN cargo fetch 
RUN cargo build --release

# runtime
FROM debian:bookworm-slim as runtime
WORKDIR /app
RUN apt-get update && apt-get install pkg-config libssl-dev curl -y

COPY --from=scraper /build/data/ ./data/
COPY --from=app-builder /app/target/release/solve ./bin/solve
COPY --from=app-builder /app/target/release/web ./bin/web

# copy css 
COPY ./konteksto-web/public/ ./konteksto-web/public/

# qdrant doesn't have cURL so we need to await manually
COPY ./scripts/deploy/wait-for-qdrant.sh .
RUN chmod +x wait-for-qdrant.sh
CMD ["./wait-for-qdrant.sh"]
