#! /usr/bin/env bash

# This script is used to initialize the database for the first time.

# Exit immediately if a command exits with a non-zero status.
set -x
set -eo pipefail

# Check does docker command exist
if ! command -v docker &> /dev/null
then
    echo "docker could not be found, please install docker first"
    exit
fi

# Check does psql command exist
if ! command -v psql &> /dev/null
then
    echo "psql could not be found, please install postgresql first"
    exit
fi

# Check does sqlx command exist
if ! command -v sqlx &> /dev/null
then
    echo "sqlx could not be found, please install sqlx-cli first"
    echo "you can install it by running: cargo install sqlx-cli"
    exit
fi

# Check if a custom user is provided or default to "postgres"
DB_USER="${POSTGRES_USER:=postgres}"

# Check if a custom password is provided or default to "password"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"

# Check if a custom database name is provided or default to "actix-template"
DB_NAME="${POSTGRES_DB:=actix-template}"

# Check if a custom port is provided or default to "5433"
DB_PORT="${POSTGRES_PORT:=5433}"

# Check if a custom host is provided or default to "localhost"
DB_HOST="${POSTGRES_HOST:=localhost}"

# Skip if a docker postgres container is already running
if ! docker ps | grep -q "postgres"; then
    # Start a postgres docker container
    docker run --rm --name postgres -e POSTGRES_PASSWORD=$DB_PASSWORD -e POSTGRES_USER=$DB_USER -e POSTGRES_DB=$DB_NAME -p $DB_PORT:5432 -d postgres postgres -N 1000
fi

# Export the password for psql
export PGPASSWORD="${DB_PASSWORD}"
# Wait for the postgres container to be ready
until psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c '\q'; do
    >&2 echo "Postgres is unavailable - sleeping"
    sleep 1
done


# Print when the postgres container is ready with port 
>&2 echo "Postgres is up and running on port $DB_PORT"


# Export the database url for sqlx
DATABASE_URL=postgres://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME
export DATABASE_URL="${DATABASE_URL}"

# Create the database 
sqlx database create

# Run the migrations
sqlx migrate run

# Print when the database is ready
>&2 echo "Finished migrations the database, lets get started!"