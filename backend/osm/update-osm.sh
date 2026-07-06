rm -rf LESSER_POLAND KRK
mkdir -p LESSER_POLAND KRK

curl -L https://download.geofabrik.de/europe/poland/malopolskie-latest.osm.pbf > LESSER_POLAND/roads.osm.pbf

osmium extract \
  --bbox=19.85,50.00,20.15,50.12 \
  -o KRK/roads.osm.pbf \
  LESSER_POLAND/roads.osm.pbf