#!/bin/bash
set -e

echo "Setting up Desmo Rust development environment..."

# Check Docker services status
echo "Checking Docker services..."
echo "✓ TimescaleDB: $(docker ps --filter "name=desmo-timescaledb" --format "{{.Status}}" | head -n1)"
echo "✓ Grafana: $(docker ps --filter "name=desmo-grafana" --format "{{.Status}}" | head -n1)"
echo "✓ NanoMQ: $(docker ps --filter "name=desmo-nanomq" --format "{{.Status}}" | head -n1)"
echo ""

# Verify Rust installation
echo "Verifying Rust installation..."
if command -v cargo >/dev/null 2>&1; then
    echo "✓ Cargo version: $(cargo --version)"
    echo "✓ Rustc version: $(rustc --version)"
    echo "✓ Rustup version: $(rustup --version)"
else
    echo "❌ Cargo not found in PATH"
    exit 1
fi

# Install additional useful Rust development tools
echo "Installing additional Rust development tools..."
cargo install cargo-expand 2>/dev/null || echo "cargo-expand already installed or installation failed"

# Verify project structure
echo "Verifying project setup..."
cd /workspace
if [ -f "Cargo.toml" ]; then
    echo "✓ Found Cargo.toml - running initial check..."
    cargo check || echo "⚠️  Initial cargo check failed - dependencies may need to be resolved"

    # Display project information
    echo ""
    echo "📋 Project Information:"
    echo "   Name: $(grep '^name = ' Cargo.toml | cut -d'"' -f2 || echo 'Unknown')"
    echo "   Version: $(grep '^version = ' Cargo.toml | cut -d'"' -f2 || echo 'Unknown')"
    echo ""

    # Show available commands
    echo "🚀 Available commands:"
    echo "   cargo build          - Build the project"
    echo "   cargo test           - Run tests"
    echo "   cargo run            - Run the MQTT to TimescaleDB bridge"
    echo "   cargo clippy         - Run linter"
    echo "   cargo fmt            - Format code"
    echo "   cargo watch -x test  - Watch and test (if cargo-watch installed)"
    echo ""

    # Desmo-specific information
    echo "📊 Desmo Bridge:"
    echo "   cargo run -- --help  - Show CLI options"
    echo "   cargo run -- --config desmo.toml  - Run with config file"
    echo ""

    # Docker compose information
    echo "🐳 Docker Services (already running):"
    echo "   TimescaleDB: localhost:5432 (user: admin, password: admin, db: metrics)"
    echo "   Grafana: http://localhost:3000 (user: admin, password: admin)"
    echo "   NanoMQ: mqtt://localhost:1883, ws://localhost:8083, http://localhost:8081"
    echo ""
else
    echo "⚠️  No Cargo.toml found - this doesn't appear to be a Rust project"
fi

echo "✅ Rust development environment setup complete!"
echo ""
echo "💡 Tips:"
echo "   - Configure desmo using desmo.toml (see desmo.toml.example)"
echo "   - All services (TimescaleDB, Grafana, NanoMQ) are running and ready"
echo "   - VS Code Rust extensions are pre-configured"
echo "   - Cargo registry is cached in volume for faster builds"
echo ""
echo "🚀 Quick start:"
echo "   1. Copy desmo.toml.example to desmo.toml"
echo "   2. Update connection settings (services are at localhost)"
echo "   3. Run: cargo run -- --config desmo.toml"
