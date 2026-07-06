bash update-data.sh
docker compose -f docker-compose.yml build --no-cache
docker compose -f docker-compose.yml up --force-recreate
