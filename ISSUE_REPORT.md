# Issue Report: Database Connection & Panic

## Status: Resolved

## Symptoms
1.  **Database Connection**: `Conn(SqlxError(PoolTimedOut))` - Application could not connect to DB.
    - Cause: Docker container was not running, and `build.rs` failed to auto-start it due to environment/path issues.
2.  **Panic on Startup**: `called Result::unwrap() on an Err value: Os { code: 2, kind: NotFound, message: "No such file or directory" }`
    - Cause: Application attempted to write to `logs/` directory which did not exist.

## Resolution
1.  **Database**:
    - Manually started database container: `docker-compose up -d`.
    - Validated connection is successful.
2.  **Logs Directory**:
    - Updated `src/logic/sync.rs` and `src/main.rs` to automatically create the `logs` directory if it doesn't exist.

## How to Run
```bash
# Ensure database is running
docker-compose up -d

# Run application
cargo run
```
