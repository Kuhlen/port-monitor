use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use port_monitor_core::features::serial::{
    DataBits, FlowControl, Parity, PortInfo, SerialConfig, SerialDataEvent, SerialErrorEvent,
    SerialSettings, StopBits, EVENT_SERIAL_DATA, EVENT_SERIAL_ERROR,
};
use port_monitor_core::AppError;
use tauri::{AppHandle, Emitter, Runtime};

const READ_TIMEOUT: Duration = Duration::from_millis(100);
const READ_BUF: usize = 1024;

// ---- serialport --------------------------------------------------------

pub fn available_ports() -> Result<Vec<PortInfo>, AppError> {
    let ports = serialport::available_ports().map_err(|e| AppError::Port(e.to_string()))?;
    Ok(ports
        .into_iter()
        .map(|p| PortInfo {
            name: p.port_name,
            port_type: describe(&p.port_type),
        })
        .collect())
}

fn describe(kind: &serialport::SerialPortType) -> String {
    match kind {
        serialport::SerialPortType::UsbPort(info) => {
            format!("USB ({})", info.product.as_deref().unwrap_or("Unknown"))
        }
        serialport::SerialPortType::BluetoothPort => "Bluetooth".to_string(),
        serialport::SerialPortType::PciPort => "PCI".to_string(),
        serialport::SerialPortType::Unknown => "Unknown".to_string(),
    }
}

pub fn open(
    config: &SerialConfig,
    settings: &SerialSettings,
) -> Result<Box<dyn serialport::SerialPort>, AppError> {
    serialport::new(&config.port, config.baud_rate)
        .data_bits(data_bits(settings.data_bits))
        .stop_bits(stop_bits(settings.stop_bits))
        .parity(parity(settings.parity))
        .flow_control(flow_control(settings.flow_control))
        .timeout(READ_TIMEOUT)
        .open()
        .map_err(|e| AppError::Port(format!("failed to open {}: {}", config.port, e)))
}

fn data_bits(v: DataBits) -> serialport::DataBits {
    match v {
        DataBits::Five => serialport::DataBits::Five,
        DataBits::Six => serialport::DataBits::Six,
        DataBits::Seven => serialport::DataBits::Seven,
        DataBits::Eight => serialport::DataBits::Eight,
    }
}

fn stop_bits(v: StopBits) -> serialport::StopBits {
    match v {
        StopBits::One => serialport::StopBits::One,
        StopBits::Two => serialport::StopBits::Two,
    }
}

fn parity(v: Parity) -> serialport::Parity {
    match v {
        Parity::None => serialport::Parity::None,
        Parity::Even => serialport::Parity::Even,
        Parity::Odd => serialport::Parity::Odd,
    }
}

fn flow_control(v: FlowControl) -> serialport::FlowControl {
    match v {
        FlowControl::None => serialport::FlowControl::None,
        FlowControl::Hardware => serialport::FlowControl::Hardware,
        FlowControl::Software => serialport::FlowControl::Software,
    }
}

// ---- reader ------------------------------------------------------------

// Blocking I/O uses a thread + stop flag, not tokio: it would end in
// spawn_blocking anyway, just via a detour.
pub fn spawn_reader<R: Runtime>(
    app: AppHandle<R>,
    mut port: Box<dyn serialport::SerialPort>,
    stop: Arc<AtomicBool>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut buf = [0u8; READ_BUF];
        let mut pending = String::new();

        while !stop.load(Ordering::Relaxed) {
            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    pending.push_str(&String::from_utf8_lossy(&buf[..n]));
                    while let Some(pos) = pending.find('\n') {
                        let line = pending[..pos].trim_end_matches('\r').to_string();
                        pending = pending[pos + 1..].to_string();
                        emit_data(&app, line);
                    }
                }
                Ok(_) => {}
                // Timeout = a pause, not an error. Flush the tail that has no
                // trailing newline so it does not hang forever.
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    if !pending.is_empty() {
                        let line = pending.trim_end_matches('\r').to_string();
                        pending.clear();
                        emit_data(&app, line);
                    }
                }
                Err(e) => {
                    let _ = app.emit(
                        EVENT_SERIAL_ERROR,
                        SerialErrorEvent {
                            message: e.to_string(),
                            timestamp: timestamp(),
                        },
                    );
                    break;
                }
            }
        }
    })
}

fn emit_data<R: Runtime>(app: &AppHandle<R>, line: String) {
    if line.is_empty() {
        return;
    }
    let _ = app.emit(
        EVENT_SERIAL_DATA,
        SerialDataEvent {
            data: line,
            timestamp: timestamp(),
        },
    );
}

fn timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S%.3f").to_string()
}

// ---- updater -----------------------------------------------------------

#[cfg(desktop)]
pub async fn check_release<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<Option<tauri_plugin_updater::Update>, AppError> {
    use tauri_plugin_updater::UpdaterExt;
    app.updater()
        .map_err(|e| AppError::Update(e.to_string()))?
        .check()
        .await
        .map_err(|e| AppError::Update(e.to_string()))
}

#[cfg(desktop)]
pub async fn install_release(update: tauri_plugin_updater::Update) -> Result<(), AppError> {
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| AppError::Update(e.to_string()))
}
