# TaskManager Development Scripts

This directory contains helper scripts for development.

## Available Scripts

### `start-db.sh`

Starts the PostgreSQL database using Docker Compose.

**Usage:**
```bash
./scripts/start-db.sh
```

This script:
- Checks for `docker-compose` (v1) or `docker compose` (v2)
- Starts the database container in detached mode
- Provides feedback on success or failure

**Note:** Make sure Docker is installed and running before executing this script.
