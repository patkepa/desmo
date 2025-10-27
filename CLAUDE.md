# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Desmo is a complete IoT telemetry monitoring solution consisting of:
- **Desmo Bridge**: Lightweight Rust application that listens to MQTT topics and automatically parses/stores telemetry data in TimescaleDB
- **Infrastructure Stack**: Docker-based services (NanoMQ MQTT broker, TimescaleDB, Grafana)

The bridge automatically detects and parses sensor readings, device logs, and raw messages from MQTT topics.

## Common Commands

### Build and Run
```bash
# Build the bridge application
cargo build --release

# Generate a configuration file
./target/release/desmo config

# Start the bridge with default config
./target/release/desmo start

# Start with custom config
./target/release/desmo start --config /path/to/config.toml

# Start with CLI overrides
./target/release/desmo start --mqtt-host localhost --mqtt-port 1883 --db-url "postgresql://admin:admin@localhost:5432/metrics"
```

### Docker Infrastructure
```bash
# Start all services
cd docker && docker compose up -d

# Check service status
docker compose ps

# View logs
docker compose logs -f
docker compose logs -f timescaledb
docker compose logs -f nanomq
docker compose logs -f grafana

# Stop services
docker compose down

# Stop and remove volumes (delete all data)
docker compose down -v
```

### Development
```bash
# Run with debug logging
RUST_LOG=debug ./target/release/desmo start

# Build for development
cargo build

# Run tests
cargo test
```

### Testing MQTT
```bash
# Subscribe to all topics
mosquitto_sub -h localhost -p 1883 -t '#' -v

# Publish test sensor reading
mosquitto_pub -h localhost -p 1883 -t 'telemetry/esp32-001/temperature' \
  -m '{"device_id": "esp32-001", "value": 25.5}'

# Publish test log
mosquitto_pub -h localhost -p 1883 -t 'diagnostics/logs/esp32-001' \
  -m '{"device_id": "esp32-001", "level": "INFO", "message": "Test log"}'
```

### Database Access
```bash
# Connect to TimescaleDB using psql
psql -h localhost -U admin -d metrics

# Or via Docker
docker exec -it cito-timescaledb psql -U admin -d metrics
```

## Architecture

### High-Level Data Flow
```
IoT Device → MQTT (NanoMQ:1883) → Desmo Bridge → TimescaleDB → Grafana Dashboards
```

### Code Structure

The Rust application is organized into four main modules:

1. **src/main.rs**: CLI entry point using clap
   - `start` command: Runs the bridge
   - `config` command: Generates sample configuration
   - Handles CLI argument parsing and config overrides

2. **src/config/mod.rs**: Configuration management
   - `Config`: Root config struct with MQTT and database settings
   - `MqttConfig`: MQTT broker connection, topics, QoS
   - `DatabaseConfig`: PostgreSQL connection URL
   - Loads from TOML files

3. **src/mqtt/mod.rs**: MQTT client and event handling
   - `MqttBridge`: Main bridge struct managing MQTT connection and event loop
   - Uses `rumqttc` for async MQTT client
   - Implements graceful shutdown via Ctrl+C handler
   - Auto-reconnects on connection errors (5s delay)
   - Processes incoming Publish packets and delegates to parser

4. **src/parser/mod.rs**: Smart message parsing logic
   - `parse_message()`: Main entry point that returns `Vec<ParsedMessage>`
   - Always stores raw payload in `socket_reads` table
   - Detects and parses three message types:
     - **Sensor readings**: JSON with `value` field, `sensors` array, or flat numeric fields
     - **Device logs**: JSON with `level`/`message` fields or plain text
     - **Plain text logs**: Automatically infers log level from topic or content
   - Extracts `device_id` from JSON or topic path
   - Extracts timestamps from JSON or uses current time

5. **src/db/mod.rs**: Database models and operations
   - Three table models: `SensorReading`, `SocketRead`, `DeviceLog`
   - Each model has an `insert()` method for database operations
   - Uses `tokio-postgres` for async database access

### Database Schema

TimescaleDB hypertables (see docker/postgres-init/init-db.sh):

- **sensor_readings**: `(timestamp, id, device_id, topic, value)` - Time-series sensor data
- **socket_reads**: `(timestamp, id, topic, payload)` - Raw MQTT payloads
- **device_logs**: `(timestamp, id, device_id, level, message, topic)` - Device logs with severity

All tables use composite primary keys `(timestamp, id)` and have indexes on device_id/topic/level.

### Message Parsing Behavior

The parser supports multiple JSON formats for flexibility:

**Single sensor value:**
```json
{"device_id": "esp32-001", "value": 25.5, "timestamp": "2025-01-15T10:30:00Z"}
```

**Multiple sensors:**
```json
{"device_id": "esp32-001", "sensors": [{"name": "temperature", "value": 25.5}]}
```

**Flat format:**
```json
{"device_id": "esp32-001", "temperature": 25.5, "humidity": 60.0}
```

**Device log:**
```json
{"device_id": "esp32-001", "level": "INFO", "message": "System initialized"}
```

**Plain text:** Any non-JSON text is parsed as a log entry with level inferred from topic/content.

### Configuration

Default MQTT topics in desmo.toml:
- `debug/diagnostics/#`
- `diagnostics/logs/+`
- `telemetry/#`

MQTT wildcards: `+` (single level), `#` (multi-level)

Default database: `postgresql://admin:admin@localhost:5432/metrics`

## Service Endpoints

- **Grafana**: http://localhost:3000 (admin/admin)
- **MQTT Broker**: localhost:1883
  - WebSocket: ws://localhost:8083/mqtt
  - HTTP API: http://localhost:8081
- **TimescaleDB**: localhost:5432 (admin/admin)

## Development Notes

- The bridge uses `tracing` for logging (set `RUST_LOG` environment variable)
- MQTT reconnection is automatic with 5-second backoff
- All messages are stored in `socket_reads` for audit trail
- The parser extracts device_id from either JSON or topic path (e.g., `telemetry/device123/temp`)
- Topics are used to create hierarchical sensor names (e.g., `telemetry/esp32/temp` → topic: `telemetry/esp32/temp/temperature`)
- Timestamps can be ISO8601 strings or Unix timestamps (seconds)

## Docker Services

- **NanoMQ**: Lightweight MQTT broker with WebSocket support and anonymous access
- **TimescaleDB**: PostgreSQL 17 with TimescaleDB extension for time-series data
- **Grafana**: Enterprise edition with pre-configured datasources for TimescaleDB

All services have health checks and automatic restart policies.

## Related Projects

This stack is designed to work with the Cito build tool telemetry features for ESP32 devices.
