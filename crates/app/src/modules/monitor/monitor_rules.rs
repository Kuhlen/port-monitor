//! pure decisions for the monitor page

use domain::AppError;
use domain::filter::LineFilter;
use domain::serial::{BaudRate, DataBits, FlowControl, Parity, PortInfo, SerialConfig, StopBits};

/// combo/segment index → option; out of range falls back to the default
pub(crate) fn pick<T: Copy + Default>(all: &[T], index: i32) -> T {
    usize::try_from(index)
        .ok()
        .and_then(|i| all.get(i))
        .copied()
        .unwrap_or_default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Frame {
    pub(crate) data_bits: DataBits,
    pub(crate) parity: Parity,
    pub(crate) stop_bits: StopBits,
    pub(crate) flow: FlowControl,
}

impl Frame {
    pub fn from_indices(data_bits: i32, parity: i32, stop_bits: i32, flow: i32) -> Self {
        Self {
            data_bits: pick(&DataBits::ALL, data_bits),
            parity: pick(&Parity::ALL, parity),
            stop_bits: pick(&StopBits::ALL, stop_bits),
            flow: pick(&FlowControl::ALL, flow),
        }
    }

    /// toolbar chip, e.g. "8N1 · no flow"
    pub fn label(&self) -> String {
        let bits = match self.data_bits {
            DataBits::Five => '5',
            DataBits::Six => '6',
            DataBits::Seven => '7',
            DataBits::Eight => '8',
        };
        let parity = match self.parity {
            Parity::None => 'N',
            Parity::Even => 'E',
            Parity::Odd => 'O',
        };
        let stop = match self.stop_bits {
            StopBits::One => '1',
            StopBits::Two => '2',
        };
        let flow = match self.flow {
            FlowControl::None => "no flow",
            FlowControl::Hardware => "RTS/CTS",
            FlowControl::Software => "XON/XOFF",
        };
        format!("{bits}{parity}{stop} · {flow}")
    }
}

pub fn config_from_form(
    ports: &[PortInfo],
    port_index: i32,
    baud: &str,
    frame: Frame,
) -> Result<SerialConfig, AppError> {
    let port = usize::try_from(port_index)
        .ok()
        .and_then(|i| ports.get(i))
        .ok_or_else(|| AppError::Port("no port selected".into()))?;
    Ok(SerialConfig {
        port: port.name.clone(),
        baud_rate: BaudRate::parse(baud)?,
        data_bits: frame.data_bits,
        parity: frame.parity,
        stop_bits: frame.stop_bits,
        flow: frame.flow,
    })
}

/// old form parity: bad skip → 0, bad keep → no limit
pub fn filter_from_form(enabled: bool, skip: &str, keep: &str, remove: &str) -> LineFilter {
    LineFilter {
        enabled,
        offset: skip.trim().parse().unwrap_or(0),
        length: keep.trim().parse().ok(),
        exclude: remove.to_string(),
    }
}

pub fn port_label(port: &PortInfo) -> String {
    format!("{} — {}", port.name, port.port_type)
}

/// rescan keeps the selected port; else first (old behavior); -1 = none
pub fn pick_port(ports: &[PortInfo], current: Option<&str>) -> i32 {
    let kept = current.and_then(|name| ports.iter().position(|p| p.name == name));
    match kept {
        Some(i) => i32::try_from(i).unwrap_or(0),
        None if ports.is_empty() => -1,
        None => 0,
    }
}

/// (title, hint) for the error banner
pub fn banner_text(error: &AppError) -> (String, String) {
    match error {
        AppError::PermissionDenied(port) => (
            format!("Can't open {port}: permission denied"),
            permission_hint().to_string(),
        ),
        AppError::Port(reason) => ("Serial port error".to_string(), reason.clone()),
        AppError::NotConnected => (
            "Not connected".to_string(),
            "Connect to a port, then try again.".to_string(),
        ),
        AppError::AlreadyConnected => (
            "Already connected".to_string(),
            "Disconnect first, then try again.".to_string(),
        ),
    }
}

fn permission_hint() -> &'static str {
    if cfg!(target_os = "linux") {
        "Add your user to the dialout group, log in again, then retry."
    } else if cfg!(windows) {
        // Windows reports a busy port as ACCESS_DENIED
        "The port is in use by another program. Close it, then retry."
    } else {
        "Access to the port was denied. Close other programs using it, then retry."
    }
}
