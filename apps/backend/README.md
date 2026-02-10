# Backend - Rust Axum Event Management API

## Overview
A high-performance REST API built with Rust, Axum, and PostgreSQL with real-time updates using PostgreSQL LISTEN/NOTIFY.

## Structure
```
apps/backend/
├── Cargo.toml              # Dependencies and project configuration
├── src/
│   ├── main.rs             # Application entry point with Axum router
│   ├── db.rs               # Database pool and LISTEN/NOTIFY listener
│   └── models.rs           # Data models (Event, Participant)
└── migrations/
    └── 001_initial_schema.sql  # Database schema with NOTIFY triggers
```

## Dependencies
- **axum** - Web framework
- **tokio** - Async runtime
- **sqlx** - SQL toolkit with compile-time query checking
- **tokio-postgres** - PostgreSQL driver for LISTEN/NOTIFY
- **serde** - Serialization/deserialization
- **tower-http** - HTTP middleware (CORS, tracing)
- **uuid** - UUID generation and handling
- **chrono** - Date/time handling

## Database Schema

### Tables
- **events** - Event information with title, description, times, location, capacity
- **participants** - Event participants with registration status

### Features
- UUID primary keys
- Automatic timestamp management (created_at, updated_at)
- Cascading deletes (delete event → delete participants)
- Status enum for participant workflow
- Indexes for performance
- LISTEN/NOTIFY triggers for real-time updates

## Setup
1. Set DATABASE_URL environment variable
2. Run migrations: `sqlx migrate run`
3. Start server: `cargo run`

## Environment Variables
- `DATABASE_URL` - PostgreSQL connection string (required)
- `PORT` - Server port (default: 3000)
- `RUST_LOG` - Log level (default: debug)

## Endpoints
- `GET /health` - Health check endpoint

## Real-time Updates
The application uses PostgreSQL LISTEN/NOTIFY to receive real-time database changes:
- `event_changes` channel - Notifies on event INSERT/UPDATE/DELETE
- `participant_changes` channel - Notifies on participant INSERT/UPDATE/DELETE

Notifications include operation type, affected ID, full data payload, and timestamp.
