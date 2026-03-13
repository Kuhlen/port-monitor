use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: String,
    pub stop_bits: String,
    pub parity: String,
    pub flow_control: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub name: String,
    pub port_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialDataEvent {
    pub data: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialErrorEvent {
    pub message: String,
    pub timestamp: String,
}
