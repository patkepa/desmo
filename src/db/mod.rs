use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use tokio_postgres::{Client, NoTls};
use tracing::{debug, error};

pub async fn connect(database_url: &str) -> Result<Client> {
    let (client, connection) = tokio_postgres::connect(database_url, NoTls)
        .await
        .with_context(|| "Failed to connect to database")?;

    // Spawn the connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Database connection error: {}", e);
        }
    });

    Ok(client)
}

#[derive(Debug)]
pub struct SensorReading {
    pub device_id: String,
    pub topic: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct SocketRead {
    pub topic: String,
    pub payload: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct DeviceLog {
    pub device_id: String,
    pub level: String,
    pub message: String,
    pub topic: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct DeviceState {
    pub device_id: String,
    pub topic: String,
    pub main_state: Option<i32>,
    pub secondary_state: Option<i32>,
    pub alerts: Option<serde_json::Value>,
    pub rssi: Option<i32>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct DeviceHealth {
    pub device_id: String,
    pub topic: String,
    pub wifi_ssid: Option<String>,
    pub free_heap_size: Option<i64>,
    pub min_heap_size: Option<i64>,
    pub unexpected_reset_counter: Option<i32>,
    pub last_reset_reason: Option<String>,
    pub wifi_connect_counter: Option<i32>,
    pub cloud_connect_counter: Option<i32>,
    pub last_wifi_connection_ts: Option<i64>,
    pub last_cloud_connection_ts: Option<i64>,
    pub timestamp: DateTime<Utc>,
}

impl SensorReading {
    pub async fn insert(&self, client: &Client) -> Result<()> {
        client
            .execute(
                "INSERT INTO sensor_readings (timestamp, device_id, topic, value) VALUES ($1, $2, $3, $4)",
                &[&self.timestamp, &self.device_id, &self.topic, &self.value],
            )
            .await
            .with_context(|| "Failed to insert sensor reading")?;

        debug!(
            "Inserted sensor reading: device={}, topic={}, value={}",
            self.device_id, self.topic, self.value
        );

        Ok(())
    }
}

impl SocketRead {
    pub async fn insert(&self, client: &Client) -> Result<()> {
        client
            .execute(
                "INSERT INTO socket_reads (timestamp, topic, payload) VALUES ($1, $2, $3)",
                &[&self.timestamp, &self.topic, &self.payload],
            )
            .await
            .with_context(|| "Failed to insert socket read")?;

        debug!("Inserted socket read: topic={}", self.topic);

        Ok(())
    }
}

impl DeviceLog {
    pub async fn insert(&self, client: &Client) -> Result<()> {
        client
            .execute(
                "INSERT INTO device_logs (timestamp, device_id, level, message, topic) VALUES ($1, $2, $3, $4, $5)",
                &[&self.timestamp, &self.device_id, &self.level, &self.message, &self.topic],
            )
            .await
            .with_context(|| "Failed to insert device log")?;

        debug!(
            "Inserted device log: device={}, level={}, message={}",
            self.device_id, self.level, self.message
        );

        Ok(())
    }
}

impl DeviceState {
    pub async fn insert(&self, client: &Client) -> Result<()> {
        // Convert alerts to JSONB - use proper JSONB format
        let alerts_json: Option<serde_json::Value> = self.alerts.clone();

        client
            .execute(
                "INSERT INTO device_states (timestamp, device_id, topic, main_state, secondary_state, alerts, rssi) VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7)",
                &[&self.timestamp, &self.device_id, &self.topic, &self.main_state, &self.secondary_state, &alerts_json, &self.rssi],
            )
            .await
            .with_context(|| format!("Failed to insert device state for device {} - timestamp: {}, main_state: {:?}, secondary_state: {:?}", self.device_id, self.timestamp, self.main_state, self.secondary_state))?;

        debug!(
            "Inserted device state: device={}, main_state={:?}, rssi={:?}",
            self.device_id, self.main_state, self.rssi
        );

        Ok(())
    }
}

impl DeviceHealth {
    pub async fn insert(&self, client: &Client) -> Result<()> {
        client
            .execute(
                "INSERT INTO device_health (timestamp, device_id, topic, wifi_ssid, free_heap_size, min_heap_size, unexpected_reset_counter, last_reset_reason, wifi_connect_counter, cloud_connect_counter, last_wifi_connection_ts, last_cloud_connection_ts) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
                &[&self.timestamp, &self.device_id, &self.topic, &self.wifi_ssid, &self.free_heap_size, &self.min_heap_size, &self.unexpected_reset_counter, &self.last_reset_reason, &self.wifi_connect_counter, &self.cloud_connect_counter, &self.last_wifi_connection_ts, &self.last_cloud_connection_ts],
            )
            .await
            .with_context(|| "Failed to insert device health")?;

        debug!(
            "Inserted device health: device={}, free_heap={:?}, reset_counter={:?}",
            self.device_id, self.free_heap_size, self.unexpected_reset_counter
        );

        Ok(())
    }
}
