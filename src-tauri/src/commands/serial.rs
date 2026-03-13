use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter, State};

use crate::state::{AppState, SerialConnection};
use crate::types::{SerialConfig, SerialDataEvent, SerialErrorEvent};

fn parse_data_bits(s: &str) -> Result<serialport::DataBits, String> {
    match s {
        "5" => Ok(serialport::DataBits::Five),
        "6" => Ok(serialport::DataBits::Six),
        "7" => Ok(serialport::DataBits::Seven),
        "8" => Ok(serialport::DataBits::Eight),
        _ => Err(format!("Invalid data bits: {s}")),
    }
}

fn parse_stop_bits(s: &str) -> Result<serialport::StopBits, String> {
    match s {
        "1" => Ok(serialport::StopBits::One),
        "2" => Ok(serialport::StopBits::Two),
        _ => Err(format!("Invalid stop bits: {s}")),
    }
}

fn parse_parity(s: &str) -> Result<serialport::Parity, String> {
    match s {
        "none" => Ok(serialport::Parity::None),
        "even" => Ok(serialport::Parity::Even),
        "odd" => Ok(serialport::Parity::Odd),
        _ => Err(format!("Invalid parity: {s}")),
    }
}

fn parse_flow_control(s: &str) -> Result<serialport::FlowControl, String> {
    match s {
        "none" => Ok(serialport::FlowControl::None),
        "hardware" => Ok(serialport::FlowControl::Hardware),
        "software" => Ok(serialport::FlowControl::Software),
        _ => Err(format!("Invalid flow control: {s}")),
    }
}

fn get_timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S%.3f").to_string()
}

#[tauri::command]
pub fn connect_port(
    config: SerialConfig,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut connection = state.lock().map_err(|e| e.to_string())?;

    if connection.is_some() {
        return Err("Already connected to a port".to_string());
    }

    let port = serialport::new(&config.port, config.baud_rate)
        .data_bits(parse_data_bits(&config.data_bits)?)
        .stop_bits(parse_stop_bits(&config.stop_bits)?)
        .parity(parse_parity(&config.parity)?)
        .flow_control(parse_flow_control(&config.flow_control)?)
        .timeout(Duration::from_millis(100))
        .open()
        .map_err(|e| format!("Failed to open port {}: {}", config.port, e))?;

    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    let thread_handle = std::thread::spawn(move || {
        let mut port = port;
        let mut buf = [0u8; 1024];
        let mut line_buf = String::new();

        while !stop_flag_clone.load(Ordering::Relaxed) {
            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    line_buf.push_str(&chunk);

                    // Emit complete lines (delimited by \n or \r\n)
                    while let Some(pos) = line_buf.find('\n') {
                        let line = line_buf[..pos].trim_end_matches('\r').to_string();
                        line_buf = line_buf[pos + 1..].to_string();

                        if !line.is_empty() {
                            let _ = app_handle.emit(
                                "serial-data",
                                SerialDataEvent {
                                    data: line,
                                    timestamp: get_timestamp(),
                                },
                            );
                        }
                    }
                }
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    // Flush remaining buffer on timeout (data without trailing newline)
                    if !line_buf.is_empty() {
                        let line = line_buf.trim_end_matches('\r').to_string();
                        line_buf.clear();

                        if !line.is_empty() {
                            let _ = app_handle.emit(
                                "serial-data",
                                SerialDataEvent {
                                    data: line,
                                    timestamp: get_timestamp(),
                                },
                            );
                        }
                    }
                }
                Err(e) => {
                    let _ = app_handle.emit(
                        "serial-error",
                        SerialErrorEvent {
                            message: e.to_string(),
                            timestamp: get_timestamp(),
                        },
                    );
                    break;
                }
            }
        }
    });

    *connection = Some(SerialConnection {
        stop_flag,
        thread_handle: Some(thread_handle),
    });

    Ok(())
}

#[tauri::command]
pub fn disconnect_port(state: State<'_, AppState>) -> Result<(), String> {
    let mut connection = state.lock().map_err(|e| e.to_string())?;

    if let Some(conn) = connection.take() {
        conn.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = conn.thread_handle {
            let _ = handle.join();
        }
        Ok(())
    } else {
        Err("Not connected to any port".to_string())
    }
}
