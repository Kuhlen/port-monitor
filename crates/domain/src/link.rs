use std::sync::mpsc::Sender;

use crate::error::AppError;
use crate::serial::{PortInfo, SerialConfig};

/// shared by data's serial_port and app's monitor_controller for LinkEvent::Line.at
pub const TIME_FORMAT: &str = "%H:%M:%S%.3f";

// traits exist for controller fakes
pub trait SerialPorts: Send + Sync {
    fn list(&self) -> Result<Vec<PortInfo>, AppError>;
    fn connect(
        &self,
        config: &SerialConfig,
        events: Sender<LinkEvent>,
    ) -> Result<Box<dyn Link>, AppError>;
}

/// dropping a Link disconnects
pub trait Link: Send {
    fn send(&self, bytes: Vec<u8>) -> Result<(), AppError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkEvent {
    Line { text: String, at: String },
    Failed(AppError),
}
