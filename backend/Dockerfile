FROM rust:trixie
WORKDIR /app
COPY ./ ./

RUN apt-get update && \
    apt-get install -y --no-install-recommends osmium-tool jq && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app/osm
RUN ./update-osm.sh

WORKDIR /app
RUN cargo build --release
EXPOSE 3000
CMD ["./target/release/tuptacz"]
