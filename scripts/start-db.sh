#!/bin/bash
# Start the PostgreSQL database using Docker Compose

set -e

echo "Starting PostgreSQL database..."

# Try docker-compose first (v1)
if command -v docker-compose &> /dev/null; then
    docker-compose up -d db
    echo "✓ Database started with docker-compose"
# Try docker compose (v2)
elif command -v docker &> /dev/null; then
    docker compose up -d db
    echo "✓ Database started with docker compose"
else
    echo "Error: Neither 'docker-compose' nor 'docker compose' command found."
    echo "Please install Docker and Docker Compose."
    exit 1
fi

echo ""
echo "Database is starting up. It may take a few seconds to be ready."
echo "Connection string: postgresql://taskmanager:password@localhost:5432/taskmanager"
