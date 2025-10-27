use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use tokio_postgres::{Client, NoTls};
use tracing::{error, info};

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

impl SensorReading {
    pub async fn insert(&self, client: &Client) -> Result<()> {
        client
            .execute(
                "INSERT INTO sensor_readings (timestamp, device_id, topic, value) VALUES ($1, $2, $3, $4)",
                &[&self.timestamp, &self.device_id, &self.topic, &self.value],
            )
            .await
            .with_context(|| "Failed to insert sensor reading")?;

        info!(
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

        info!("Inserted socket read: topic={}", self.topic);

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

        info!(
            "Inserted device log: device={}, level={}, message={}",
            self.device_id, self.level, self.message
        );

        Ok(())
    }
}
