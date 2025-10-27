use chrono::Utc;
use serde_json::Value;
use tracing::{debug, warn};

use crate::db::{DeviceLog, SensorReading, SocketRead};

/// Parse MQTT message into database records
pub fn parse_message(topic: &str, payload: &[u8]) -> Vec<ParsedMessage> {
    let mut results = Vec::new();

    // Convert payload to string
    let payload_str = match String::from_utf8(payload.to_vec()) {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to decode payload as UTF-8: {}", e);
            return results;
        }
    };

    // Always store raw message
    results.push(ParsedMessage::SocketRead(SocketRead {
        topic: topic.to_string(),
        payload: payload_str.clone(),
        timestamp: Utc::now(),
    }));

    // Try to parse as JSON
    if let Ok(json) = serde_json::from_str::<Value>(&payload_str) {
        // Parse sensor readings
        if let Some(readings) = parse_sensor_readings(topic, &json) {
            results.extend(readings.into_iter().map(ParsedMessage::SensorReading));
        }

        // Parse device logs
        if let Some(log) = parse_device_log(topic, &json) {
            results.push(ParsedMessage::DeviceLog(log));
        }
    } else {
        // Try to parse as plain text log
        if let Some(log) = parse_plain_text_log(topic, &payload_str) {
            results.push(ParsedMessage::DeviceLog(log));
        }
    }

    debug!("Parsed {} records from topic {}", results.len(), topic);
    results
}

#[derive(Debug)]
pub enum ParsedMessage {
    SensorReading(SensorReading),
    SocketRead(SocketRead),
    DeviceLog(DeviceLog),
}

/// Parse JSON sensor readings
fn parse_sensor_readings(topic: &str, json: &Value) -> Option<Vec<SensorReading>> {
    let mut readings = Vec::new();

    // Extract device_id from topic or JSON
    let device_id = extract_device_id(topic, json)?;

    // Handle single sensor value
    if let Some(value) = json.get("value").and_then(|v| v.as_f64()) {
        readings.push(SensorReading {
            device_id: device_id.clone(),
            topic: topic.to_string(),
            value,
            timestamp: extract_timestamp(json),
        });
    }

    // Handle multiple sensor values in "sensors" array
    if let Some(sensors) = json.get("sensors").and_then(|v| v.as_array()) {
        for sensor in sensors {
            if let (Some(name), Some(value)) = (
                sensor.get("name").and_then(|v| v.as_str()),
                sensor.get("value").and_then(|v| v.as_f64()),
            ) {
                readings.push(SensorReading {
                    device_id: device_id.clone(),
                    topic: format!("{}/{}", topic, name),
                    value,
                    timestamp: extract_timestamp(json),
                });
            }
        }
    }

    // Handle flat JSON with numeric values (e.g., {"temperature": 25.5, "humidity": 60.0})
    if let Some(obj) = json.as_object() {
        for (key, value) in obj {
            if let Some(num) = value.as_f64() {
                if key != "timestamp" && key != "device_id" {
                    readings.push(SensorReading {
                        device_id: device_id.clone(),
                        topic: format!("{}/{}", topic, key),
                        value: num,
                        timestamp: extract_timestamp(json),
                    });
                }
            }
        }
    }

    if readings.is_empty() {
        None
    } else {
        Some(readings)
    }
}

/// Parse device log from JSON
fn parse_device_log(topic: &str, json: &Value) -> Option<DeviceLog> {
    // Check if this looks like a log message
    let level = json
        .get("level")
        .or_else(|| json.get("severity"))
        .and_then(|v| v.as_str())?;

    let message = json
        .get("message")
        .or_else(|| json.get("msg"))
        .or_else(|| json.get("text"))
        .and_then(|v| v.as_str())?;

    let device_id = extract_device_id(topic, json)?;

    Some(DeviceLog {
        device_id,
        level: level.to_string(),
        message: message.to_string(),
        topic: topic.to_string(),
        timestamp: extract_timestamp(json),
    })
}

/// Parse plain text log
fn parse_plain_text_log(topic: &str, text: &str) -> Option<DeviceLog> {
    // Extract device_id from topic
    let device_id = topic
        .split('/')
        .find(|part| !part.is_empty() && *part != "diagnostics" && *part != "debug" && *part != "logs")
        .unwrap_or("unknown")
        .to_string();

    // Determine log level from topic or content
    let level = if topic.contains("error") || text.to_lowercase().contains("error") {
        "ERROR"
    } else if topic.contains("warn") || text.to_lowercase().contains("warn") {
        "WARN"
    } else if topic.contains("debug") || topic.contains("diagnostics") {
        "DEBUG"
    } else {
        "INFO"
    };

    Some(DeviceLog {
        device_id,
        level: level.to_string(),
        message: text.to_string(),
        topic: topic.to_string(),
        timestamp: Utc::now(),
    })
}

/// Extract device_id from topic or JSON
fn extract_device_id(topic: &str, json: &Value) -> Option<String> {
    // Try to get from JSON first
    if let Some(id) = json
        .get("device_id")
        .or_else(|| json.get("deviceId"))
        .or_else(|| json.get("device"))
        .and_then(|v| v.as_str())
    {
        return Some(id.to_string());
    }

    // Try to extract from topic (e.g., "telemetry/device123/temperature")
    let parts: Vec<&str> = topic.split('/').collect();
    if parts.len() >= 2 {
        // Look for part that looks like a device ID
        for part in parts.iter() {
            if part.starts_with("device") || part.len() >= 8 {
                return Some(part.to_string());
            }
        }
    }

    Some("unknown".to_string())
}

/// Extract timestamp from JSON or use current time
fn extract_timestamp(json: &Value) -> chrono::DateTime<Utc> {
    if let Some(ts) = json.get("timestamp").or_else(|| json.get("ts")) {
        // Try to parse as ISO8601 string
        if let Some(ts_str) = ts.as_str() {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts_str) {
                return dt.with_timezone(&Utc);
            }
        }

        // Try to parse as Unix timestamp (seconds)
        if let Some(ts_num) = ts.as_i64() {
            if let Some(dt) = chrono::DateTime::from_timestamp(ts_num, 0) {
                return dt;
            }
        }
    }

    Utc::now()
}
