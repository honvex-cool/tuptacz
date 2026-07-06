FROM rust:trixie
WORKDIR /app
COPY ./ ./

RUN apt-get update && \
    apt-get install -y --no-install-recommends osmium-tool jq && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app/osm
RUN ./update-osm.sh

WORKDIR /app
EXPOSE 3000
CMD ["cargo", "run"]
