FROM ghcr.io/astral-sh/uv:debian-slim AS scraper

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

WORKDIR /build

COPY builder ./builder
COPY scripts/build-db.sh .

RUN chmod +x build-db.sh
RUN ./build-db.sh

FROM rust:slim-bookworm as app-builder
# do nothing here with the --from vector-builder
