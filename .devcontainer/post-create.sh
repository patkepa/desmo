#!/bin/bash
set -e

echo "Setting up Desmo Rust development environment..."

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
    if [ -d "docker" ]; then
        echo "🐳 Docker Compose Services:"
        echo "   cd docker && docker-compose up  - Start TimescaleDB, Grafana, and NanoMQ"
        echo ""
    fi
else
    echo "⚠️  No Cargo.toml found - this doesn't appear to be a Rust project"
fi

echo "✅ Rust development environment setup complete!"
echo ""
echo "💡 Tips:"
echo "   - Configure desmo using desmo.toml (see desmo.toml.example)"
echo "   - VS Code Rust extensions are pre-configured"
echo "   - Cargo registry is cached in volume for faster builds"
