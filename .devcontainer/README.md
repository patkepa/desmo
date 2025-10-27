# Desmo DevContainer

Simplified Rust development environment for the Desmo MQTT to TimescaleDB bridge.

## Quick Start

```bash
# First time (builds container - takes 2-3 minutes)
devpod up . --ide none

# Connect to container
devpod ssh .

# Inside container - build Desmo
cargo build

# Use Desmo Bridge
cargo run -- --help                  # Show help
cargo run -- --config desmo.toml     # Run with config file
```

## What's Included

- **Rust 1.89** - Latest stable Rust toolchain
- **Development tools** - clippy, rustfmt, cargo-watch
- **System dependencies** - Required libraries for Desmo compilation (OpenSSL, etc.)

## Daily Usage

```bash
# Connect (instant if container exists)
devpod ssh .

# Stop container (preserves it)
devpod stop .

# Restart stopped container (few seconds)
devpod up . --ide none

# Only rebuild when Dockerfile changes
devpod up . --ide none --recreate
```

## Container Architecture

- **Desmo Bridge** - Lightweight MQTT to TimescaleDB bridge
- **Async Runtime** - Uses Tokio for high-performance async I/O
- **MQTT Client** - rumqttc for MQTT connections
- **PostgreSQL** - tokio-postgres for TimescaleDB integration
- Persistent volumes for faster rebuilds

## Local Development Setup

For full stack development with TimescaleDB, Grafana, and NanoMQ:

1. Start the devcontainer: `devpod up . --ide none`
2. Connect: `devpod ssh .`
3. In a separate terminal (on host), start Docker services:
   ```bash
   cd docker
   docker-compose up
   ```
4. Inside devcontainer, configure and run Desmo:
   ```bash
   cp desmo.toml.example desmo.toml
   # Edit desmo.toml with your settings
   cargo run -- --config desmo.toml
   ```

## Configuration

- Copy `desmo.toml.example` to `desmo.toml`
- Configure MQTT broker settings
- Configure TimescaleDB connection
- Set up topic mappings and data parsing rules

## Troubleshooting

- **First build slow?** Normal - only happens once
- **Can't connect?** Try `devpod delete . --force` then `devpod up .`
- **Database connection issues?** Ensure Docker Compose services are running on the host
