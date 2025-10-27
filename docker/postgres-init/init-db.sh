#!/bin/bash
set -e

# Create the TimescaleDB extension
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

    -- Create tables
    CREATE TABLE IF NOT EXISTS sensor_readings (
        timestamp TIMESTAMPTZ NOT NULL,
        id SERIAL NOT NULL,
        device_id TEXT NOT NULL,
        topic TEXT NOT NULL,
        value DOUBLE PRECISION NOT NULL,
        PRIMARY KEY (timestamp, id)
    );

    CREATE TABLE IF NOT EXISTS socket_reads (
        timestamp TIMESTAMPTZ NOT NULL,
        id SERIAL NOT NULL,
        topic TEXT NOT NULL,
        payload TEXT NOT NULL,
        PRIMARY KEY (timestamp, id)
    );

    CREATE TABLE IF NOT EXISTS device_logs (
        timestamp TIMESTAMPTZ NOT NULL,
        id SERIAL NOT NULL,
        device_id TEXT NOT NULL,
        level TEXT NOT NULL,
        message TEXT NOT NULL,
        topic TEXT NOT NULL,
        PRIMARY KEY (timestamp, id)
    );

    CREATE TABLE IF NOT EXISTS device_states (
        timestamp TIMESTAMPTZ NOT NULL,
        id SERIAL NOT NULL,
        device_id TEXT NOT NULL,
        topic TEXT NOT NULL,
        main_state INTEGER,
        secondary_state INTEGER,
        alerts JSONB,
        rssi INTEGER,
        PRIMARY KEY (timestamp, id)
    );

    CREATE TABLE IF NOT EXISTS device_health (
        timestamp TIMESTAMPTZ NOT NULL,
        id SERIAL NOT NULL,
        device_id TEXT NOT NULL,
        topic TEXT NOT NULL,
        wifi_ssid TEXT,
        free_heap_size BIGINT,
        min_heap_size BIGINT,
        unexpected_reset_counter INTEGER,
        last_reset_reason TEXT,
        wifi_connect_counter INTEGER,
        cloud_connect_counter INTEGER,
        last_wifi_connection_ts BIGINT,
        last_cloud_connection_ts BIGINT,
        PRIMARY KEY (timestamp, id)
    );

    -- Convert to hypertables
    SELECT create_hypertable('sensor_readings', 'timestamp', if_not_exists => TRUE);
    SELECT create_hypertable('socket_reads', 'timestamp', if_not_exists => TRUE);
    SELECT create_hypertable('device_logs', 'timestamp', if_not_exists => TRUE);
    SELECT create_hypertable('device_states', 'timestamp', if_not_exists => TRUE);
    SELECT create_hypertable('device_health', 'timestamp', if_not_exists => TRUE);

    -- Create indexes
    CREATE INDEX IF NOT EXISTS idx_sensor_readings_device_id ON sensor_readings (device_id);
    CREATE INDEX IF NOT EXISTS idx_socket_reads_topic ON socket_reads (topic);
    CREATE INDEX IF NOT EXISTS idx_device_logs_device_id ON device_logs (device_id);
    CREATE INDEX IF NOT EXISTS idx_device_logs_level ON device_logs (level);
    CREATE INDEX IF NOT EXISTS idx_device_states_device_id ON device_states (device_id);
    CREATE INDEX IF NOT EXISTS idx_device_health_device_id ON device_health (device_id);

    -- Configure proper authentication
    ALTER USER admin WITH PASSWORD 'admin';
EOSQL

# Update pg_hba.conf to allow connections from the network
cat > "${PGDATA}/pg_hba.conf" <<EOF
# TYPE  DATABASE        USER            ADDRESS                 METHOD
local   all             all                                     trust
host    all             all             127.0.0.1/32            md5
host    all             all             ::1/128                 md5
host    all             all             0.0.0.0/0               md5
EOF

chmod 600 "${PGDATA}/pg_hba.conf"
