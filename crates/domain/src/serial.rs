use std::fmt;

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortInfo {
    pub name: String,
    pub port_type: String,
}

/// > 0; the only way in is parse
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BaudRate(u32);

impl BaudRate {
    pub fn parse(raw: &str) -> Result<Self, AppError> {
        raw.trim()
            .parse::<u32>()
            .ok()
            .filter(|&b| b > 0)
            .map(Self)
            .ok_or_else(|| AppError::Port(format!("invalid baud rate: {raw}")))
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for BaudRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

// typed fields: the old String wire format and its validate() are gone
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: BaudRate,
    pub data_bits: DataBits,
    pub parity: Parity,
    pub stop_bits: StopBits,
    pub flow: FlowControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataBits {
    Five,
    Six,
    Seven,
    #[default]
    Eight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Parity {
    #[default]
    None,
    Even,
    Odd,
}

// 1.5 dropped: serialport never accepted it (spec D6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StopBits {
    #[default]
    One,
    Two,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlowControl {
    #[default]
    None,
    Hardware,
    Software,
}

// ALL order = segment order in frame_popup.slint
impl DataBits {
    pub const ALL: [Self; 4] = [Self::Five, Self::Six, Self::Seven, Self::Eight];
}
impl Parity {
    pub const ALL: [Self; 3] = [Self::None, Self::Even, Self::Odd];
}
impl StopBits {
    pub const ALL: [Self; 2] = [Self::One, Self::Two];
}
impl FlowControl {
    pub const ALL: [Self; 3] = [Self::None, Self::Hardware, Self::Software];
}
