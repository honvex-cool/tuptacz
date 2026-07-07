echo "Downloading OSM data"

rm -rf LESSER_POLAND KRK OLD_TOWN DEBNIKI
mkdir -p LESSER_POLAND KRK OLD_TOWN DEBNIKI

echo "Downloading LESSER_POLAND"
curl -L 'https://download.geofabrik.de/europe/poland/malopolskie-latest.osm.pbf' > LESSER_POLAND/roads.osm.pbf
curl -L 'https://nominatim.openstreetmap.org/lookup?osm_ids=R224459&polygon_geojson=1&format=json' | jq '{"type":"Feature","geometry":.[0].geojson,"properties":{}}' > LESSER_POLAND/polygon.json

echo "Downloading and extracting KRK"
curl -L 'https://nominatim.openstreetmap.org/lookup?osm_ids=R449696&polygon_geojson=1&format=json' | jq '{"type":"Feature","geometry":.[0].geojson,"properties":{}}' > KRK/polygon.json
osmium extract --polygon KRK/polygon.json -o KRK/roads.osm.pbf LESSER_POLAND/roads.osm.pbf

echo "Downloading and extracting OLD_TOWN"
curl -L 'https://nominatim.openstreetmap.org/lookup?osm_ids=R2642241&polygon_geojson=1&format=json' | jq '{"type":"Feature","geometry":.[0].geojson,"properties":{}}' > OLD_TOWN/polygon.json
osmium extract --polygon OLD_TOWN/polygon.json -o OLD_TOWN/roads.osm.pbf LESSER_POLAND/roads.osm.pbf

echo "Downloading and extracting DEBNIKI"
curl -L 'https://nominatim.openstreetmap.org/lookup?osm_ids=R2398482&polygon_geojson=1&format=json' | jq '{"type":"Feature","geometry":.[0].geojson,"properties":{}}' > DEBNIKI/polygon.json
osmium extract --polygon DEBNIKI/polygon.json -o DEBNIKI/roads.osm.pbf LESSER_POLAND/roads.osm.pbf
