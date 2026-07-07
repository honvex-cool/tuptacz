bash stop-dev.sh
bash update-data.sh
docker compose -f docker-compose.dev.yml build
docker compose -f docker-compose.dev.yml up
