//! serialport impl. one thread per link reads and writes; stop flag, no tokio.

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use domain::AppError;
use domain::line_buffer::LineBuffer;
use domain::link::{Link, LinkEvent, SerialPorts};
use domain::serial::{DataBits, FlowControl, Parity, PortInfo, SerialConfig, StopBits};
use serialport::SerialPort;

// also decides when a newline-less tail is flushed
const READ_TIMEOUT: Duration = Duration::from_millis(100);
const READ_BUF: usize = 1024;

#[derive(Debug)]
pub struct SerialPortLink;

impl SerialPorts for SerialPortLink {
    fn list(&self) -> Result<Vec<PortInfo>, AppError> {
        let ports = serialport::available_ports().map_err(|e| AppError::Port(e.to_string()))?;
        Ok(ports
            .into_iter()
            .map(|p| PortInfo {
                name: p.port_name,
                port_type: describe(&p.port_type),
            })
            .collect())
    }

    fn connect(
        &self,
        config: &SerialConfig,
        events: Sender<LinkEvent>,
    ) -> Result<Box<dyn Link>, AppError> {
        let port = serialport::new(&config.port, config.baud_rate.get())
            .data_bits(data_bits(config.data_bits))
            .parity(parity(config.parity))
            .stop_bits(stop_bits(config.stop_bits))
            .flow_control(flow_control(config.flow))
            .timeout(READ_TIMEOUT)
            .open()
            .map_err(|e| open_error(&config.port, e))?;
        Ok(Box::new(SerialLink::spawn(port, events)))
    }
}

fn open_error(port: &str, e: serialport::Error) -> AppError {
    let denied = match e.kind() {
        serialport::ErrorKind::Io(io::ErrorKind::PermissionDenied) => true,
        // Windows maps ACCESS_DENIED and FILE_NOT_FOUND both to NoDevice: listed port = busy
        // (also mislabels a listed-but-powered-off Bluetooth port as busy — accepted)
        serialport::ErrorKind::NoDevice if cfg!(windows) => listed(port),
        _ => false,
    };
    if denied {
        AppError::PermissionDenied(port.to_string())
    } else {
        AppError::Port(format!("failed to open {port}: {e}"))
    }
}

fn listed(port: &str) -> bool {
    serialport::available_ports().is_ok_and(|ports| ports.iter().any(|p| p.port_name == port))
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

pub struct SerialLink {
    outbox: Sender<Vec<u8>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl SerialLink {
    /// public for pty tests; SerialPortLink::connect is the real entry
    pub fn spawn(mut port: Box<dyn SerialPort>, events: Sender<LinkEvent>) -> Self {
        let (outbox, inbox) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let thread = std::thread::spawn(move || run(port.as_mut(), &inbox, &events, &flag));
        Self {
            outbox,
            stop,
            thread: Some(thread),
        }
    }
}

impl Link for SerialLink {
    fn send(&self, bytes: Vec<u8>) -> Result<(), AppError> {
        // closed channel = thread exited = link dead
        self.outbox.send(bytes).map_err(|_| AppError::NotConnected)
    }
}

impl Drop for SerialLink {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            // ponytail: join also waits for the OS close, which can take seconds on Windows Bluetooth COM ports; detach with a timeout if users hit it
            let _ = thread.join();
        }
    }
}

// ponytail: one thread reads and writes, so a send waits ≤ 100 ms on an idle line;
// writer thread over try_clone() if latency ever matters
fn run(
    port: &mut dyn SerialPort,
    inbox: &Receiver<Vec<u8>>,
    events: &Sender<LinkEvent>,
    stop: &AtomicBool,
) {
    let mut buf = [0u8; READ_BUF];
    let mut lines = LineBuffer::default();
    while !stop.load(Ordering::Relaxed) {
        if let Err(e) = write_pending(port, inbox) {
            return fail(events, &mut lines, &e);
        }
        match port.read(&mut buf) {
            Ok(n) => lines
                .push(&buf[..n])
                .into_iter()
                .for_each(|text| emit(events, text)),
            // timeout = pause, not error: flush the tail so it does not hang
            Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                if let Some(text) = lines.flush() {
                    emit(events, text);
                }
            }
            Err(e) => return fail(events, &mut lines, &e),
        }
    }
}

fn write_pending(port: &mut dyn SerialPort, inbox: &Receiver<Vec<u8>>) -> io::Result<()> {
    for bytes in inbox.try_iter() {
        port.write_all(&bytes)?;
        // no flush(): tcdrain has no timeout, would hang Drop's join under RTS/CTS
    }
    Ok(())
}

// receiver gone = UI already dropped this link; nothing to tell
fn emit(events: &Sender<LinkEvent>, text: String) {
    let _ = events.send(LinkEvent::Line {
        text,
        at: timestamp(),
    });
}

// flush the tail first: unplug mid-line must not lose it
fn fail(events: &Sender<LinkEvent>, lines: &mut LineBuffer, e: &io::Error) {
    if let Some(text) = lines.flush() {
        emit(events, text);
    }
    let _ = events.send(LinkEvent::Failed(AppError::Port(e.to_string())));
}

fn timestamp() -> String {
    chrono::Local::now()
        .format(domain::link::TIME_FORMAT)
        .to_string()
}
