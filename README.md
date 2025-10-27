# Desmo - Device Telemetry Monitoring Stack

Desmo is a complete telemetry monitoring solution for IoT devices, featuring MQTT message collection, TimescaleDB storage, and Grafana visualization.

## Components

### Infrastructure (Docker)
- **NanoMQ**: Lightweight MQTT broker for device communication
- **TimescaleDB**: PostgreSQL-based time-series database
- **Grafana**: Real-time dashboards and visualization

### Desmo Bridge (Rust Application)
- **MQTT to Database Bridge**: Lightweight Rust application that listens to MQTT topics and automatically parses/stores telemetry data in TimescaleDB
- **Smart Message Parsing**: Automatically detects and parses sensor readings, device logs, and raw messages
- **Real-time Processing**: Low latency message processing with automatic reconnection

## Quick Start

### Prerequisites
- Docker and Docker Compose installed
- Rust toolchain (for building the bridge)
- Ports 1883 (MQTT), 3000 (Grafana), and 5432 (PostgreSQL) available

### 1. Start the Infrastructure

```bash
cd docker
docker compose up -d
```

Wait for all services to be healthy (about 30 seconds):
```bash
docker compose ps
```

### 2. Build and Run the Desmo Bridge

```bash
# Build the bridge application
cargo build --release

# Generate a configuration file
./target/release/desmo-bridge config

# Edit desmo.toml if needed, then start the bridge
./target/release/desmo-bridge start
```

Alternatively, with custom settings:
```bash
./target/release/desmo-bridge start --mqtt-host localhost --mqtt-port 1883 --db-url "postgresql://admin:admin@localhost:5432/metrics"
```

### Access the Services

- **Grafana**: http://localhost:3000
  - Username: `admin`
  - Password: `admin`

- **MQTT Broker**: `localhost:1883`
  - WebSocket: `ws://localhost:8083/mqtt`
  - HTTP API: http://localhost:8081

- **TimescaleDB**: `localhost:5432`
  - Database: `metrics`
  - Username: `admin`
  - Password: `admin`

### View Logs

```bash
docker compose logs -f
```

### Stop the Stack

```bash
docker compose down
```

To remove volumes (delete all data):
```bash
docker compose down -v
```

## Architecture

### Services

#### NanoMQ MQTT Broker
- Lightweight, high-performance MQTT broker
- Supports MQTT, WebSocket, and HTTP API
- Anonymous access enabled for easy development

#### TimescaleDB
- PostgreSQL 17 with TimescaleDB extension
- Optimized for time-series data storage
- Pre-configured with three hypertables:
  - `sensor_readings`: Device sensor data (device_id, topic, value, timestamp)
  - `socket_reads`: Raw MQTT message payloads
  - `device_logs`: Device log messages with severity levels

#### Grafana
- Enterprise edition with alerting features
- Pre-configured datasources for TimescaleDB
- Anonymous access enabled (Admin role)

### Data Flow

```
IoT Device → MQTT (NanoMQ:1883) → Application → TimescaleDB → Grafana Dashboards
```

## Database Schema

### sensor_readings
```sql
CREATE TABLE sensor_readings (
    timestamp TIMESTAMPTZ NOT NULL,
    id SERIAL NOT NULL,
    device_id TEXT NOT NULL,
    topic TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    PRIMARY KEY (timestamp, id)
);
```

### socket_reads
```sql
CREATE TABLE socket_reads (
    timestamp TIMESTAMPTZ NOT NULL,
    id SERIAL NOT NULL,
    topic TEXT NOT NULL,
    payload TEXT NOT NULL,
    PRIMARY KEY (timestamp, id)
);
```

### device_logs
```sql
CREATE TABLE device_logs (
    timestamp TIMESTAMPTZ NOT NULL,
    id SERIAL NOT NULL,
    device_id TEXT NOT NULL,
    level TEXT NOT NULL,
    message TEXT NOT NULL,
    topic TEXT NOT NULL,
    PRIMARY KEY (timestamp, id)
);
```

## Configuration

### MQTT Topics
By default, the system is designed to work with topics like:
- `debug/diagnostics/#` - Debug and diagnostic messages
- `diagnostics/logs/+` - Device log messages
- Custom topics can be added as needed

### Grafana Datasources
Two datasources are pre-configured:
- **TimescaleDB** (default): `metrics` database
- **TelemetryDB**: `telemetry_data` database (if needed)

### Environment Variables
All services can be customized via environment variables in `docker-compose.yml`:
- Database credentials
- MQTT broker settings
- Grafana authentication settings

## Desmo Bridge

### Configuration

The bridge uses a TOML configuration file (default: `desmo.toml`):

```toml
[mqtt]
host = "localhost"
port = 1883
client_id = "desmo-bridge"
qos = 0
topics = [
    "debug/diagnostics/#",
    "diagnostics/logs/+",
    "telemetry/#",
]

[database]
url = "postgresql://admin:admin@localhost:5432/metrics"
```

### Message Parsing

The bridge automatically parses different message formats:

#### Sensor Readings (JSON)
```json
{
  "device_id": "esp32-001",
  "value": 25.5,
  "timestamp": "2025-01-15T10:30:00Z"
}
```

Or multiple sensors:
```json
{
  "device_id": "esp32-001",
  "sensors": [
    {"name": "temperature", "value": 25.5},
    {"name": "humidity", "value": 60.0}
  ]
}
```

Or flat format:
```json
{
  "device_id": "esp32-001",
  "temperature": 25.5,
  "humidity": 60.0
}
```

#### Device Logs (JSON)
```json
{
  "device_id": "esp32-001",
  "level": "INFO",
  "message": "System initialized successfully"
}
```

#### Plain Text Logs
Any plain text message is automatically parsed as a log entry with level inferred from topic or content.

### CLI Usage

```bash
# Start with default config
desmo-bridge start

# Start with custom config file
desmo-bridge start --config /path/to/config.toml

# Override config via CLI
desmo-bridge start --mqtt-host 192.168.1.100 --mqtt-port 1883

# Generate sample config
desmo-bridge config --output my-config.toml
```

### Environment Variables

Set `RUST_LOG` for detailed logging:
```bash
RUST_LOG=debug desmo-bridge start
```

## Development

### Testing MQTT Connection

```bash
# Subscribe to all topics
mosquitto_sub -h localhost -p 1883 -t '#' -v

# Publish a test sensor reading
mosquitto_pub -h localhost -p 1883 -t 'telemetry/esp32-001/temperature' \
  -m '{"device_id": "esp32-001", "value": 25.5}'

# Publish a test log
mosquitto_pub -h localhost -p 1883 -t 'diagnostics/logs/esp32-001' \
  -m '{"device_id": "esp32-001", "level": "INFO", "message": "Test log"}'
```

### Connecting to TimescaleDB

```bash
# Using psql
psql -h localhost -U admin -d metrics

# Or using Docker
docker exec -it cito-timescaledb psql -U admin -d metrics
```

### Grafana Dashboard Development
1. Access Grafana at http://localhost:3000
2. Navigate to Dashboards → New Dashboard
3. Select TimescaleDB as datasource
4. Query the hypertables for time-series data

## Troubleshooting

### Check Service Health
```bash
docker compose ps
```

### View Service Logs
```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f timescaledb
docker compose logs -f grafana
docker compose logs -f nanomq
```

### Reset Everything
```bash
# Stop and remove containers, networks, and volumes
docker compose down -v

# Start fresh
docker compose up -d
```

### MQTT Connection Issues
- Verify NanoMQ is running: `docker compose ps nanomq`
- Check NanoMQ logs: `docker compose logs nanomq`
- Test with mosquitto_pub/sub tools

### Database Connection Issues
- Verify TimescaleDB health: `docker exec cito-timescaledb pg_isready -U admin`
- Check initialization logs: `docker compose logs timescaledb | grep -i error`

## Production Considerations

Before deploying to production:

1. **Change default passwords** in `docker-compose.yml`
2. **Disable anonymous access** in Grafana
3. **Enable MQTT authentication** in NanoMQ
4. **Configure SSL/TLS** for all services
5. **Set up proper backup strategy** for TimescaleDB volumes
6. **Configure resource limits** for containers
7. **Implement monitoring and alerting**

## Integration with Cito

This stack is designed to work with the [Cito build tool](../README.md) telemetry features:

```bash
# Start receiving telemetry from ESP32 devices
cito telemetry start --host localhost --port 1883 --topics "debug/diagnostics/#,diagnostics/logs/+"
```

## License

See [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or pull request.
