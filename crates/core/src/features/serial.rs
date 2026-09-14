use serde::{Deserialize, Serialize};

use crate::error::AppError;

// Event names used by backend (emit) and frontend (listen). One definition,
// so they cannot drift into a dead string.
pub const EVENT_SERIAL_DATA: &str = "serial-data";
pub const EVENT_SERIAL_ERROR: &str = "serial-error";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortInfo {
    pub name: String,
    pub port_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerialDataEvent {
    pub data: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerialErrorEvent {
    pub message: String,
    pub timestamp: String,
}

// Wire format stays String: that is what the UI <select> binds to. Legal
// values are pinned by the enums below, not by a silent two-side handshake.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: String,
    pub stop_bits: String,
    pub parity: String,
    pub flow_control: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataBits {
    Five,
    Six,
    Seven,
    Eight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopBits {
    One,
    Two,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parity {
    None,
    Even,
    Odd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowControl {
    None,
    Hardware,
    Software,
}

impl DataBits {
    pub fn parse(s: &str) -> Result<Self, AppError> {
        match s {
            "5" => Ok(Self::Five),
            "6" => Ok(Self::Six),
            "7" => Ok(Self::Seven),
            "8" => Ok(Self::Eight),
            _ => Err(AppError::Validation(format!("data bits: {s}"))),
        }
    }
}

impl StopBits {
    pub fn parse(s: &str) -> Result<Self, AppError> {
        match s {
            "1" => Ok(Self::One),
            "2" => Ok(Self::Two),
            _ => Err(AppError::Validation(format!("stop bits: {s}"))),
        }
    }
}

impl Parity {
    pub fn parse(s: &str) -> Result<Self, AppError> {
        match s {
            "none" => Ok(Self::None),
            "even" => Ok(Self::Even),
            "odd" => Ok(Self::Odd),
            _ => Err(AppError::Validation(format!("parity: {s}"))),
        }
    }
}

impl FlowControl {
    pub fn parse(s: &str) -> Result<Self, AppError> {
        match s {
            "none" => Ok(Self::None),
            "hardware" => Ok(Self::Hardware),
            "software" => Ok(Self::Software),
            _ => Err(AppError::Validation(format!("flow control: {s}"))),
        }
    }
}

// Parse result feeds infra's mapping to serialport. Frontend discards it,
// only wants the Err.
pub struct SerialSettings {
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
}

impl SerialConfig {
    // Called on BOTH sides: frontend for instant feedback, backend as the
    // real gate.
    pub fn validate(&self) -> Result<SerialSettings, AppError> {
        if self.port.trim().is_empty() {
            return Err(AppError::Validation("empty port".into()));
        }
        if self.baud_rate == 0 {
            return Err(AppError::Validation("zero baud rate".into()));
        }
        Ok(SerialSettings {
            data_bits: DataBits::parse(&self.data_bits)?,
            stop_bits: StopBits::parse(&self.stop_bits)?,
            parity: Parity::parse(&self.parity)?,
            flow_control: FlowControl::parse(&self.flow_control)?,
        })
    }
}

// No Send bound, deliberate. Never dyn'd, so the compiler sees each impl's
// concrete future.
#[allow(async_fn_in_trait)]
pub trait SerialApi {
    async fn list_ports(&self) -> Result<Vec<PortInfo>, AppError>;
    async fn connect_port(&self, config: SerialConfig) -> Result<(), AppError>;
    async fn disconnect_port(&self) -> Result<(), AppError>;
}
