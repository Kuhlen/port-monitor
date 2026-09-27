use std::sync::mpsc;

use data::serial_port::SerialPortLink;
use domain::AppError;
use domain::link::SerialPorts;
use domain::serial::{BaudRate, DataBits, FlowControl, Parity, SerialConfig, StopBits};

fn config(port: &str) -> SerialConfig {
    SerialConfig {
        port: port.into(),
        baud_rate: BaudRate::parse("9600").expect("baud"),
        data_bits: DataBits::Eight,
        parity: Parity::None,
        stop_bits: StopBits::One,
        flow: FlowControl::None,
    }
}

#[test]
fn missing_port_is_port_error() {
    let port = if cfg!(windows) {
        "COM99"
    } else {
        "/dev/nonexistent-pm"
    };
    let (tx, _rx) = mpsc::channel();
    let result = SerialPortLink.connect(&config(port), tx);
    assert!(
        matches!(result, Err(AppError::Port(_))),
        "{:?}",
        result.err()
    );
}

#[test]
fn list_does_not_fail() {
    SerialPortLink.list().expect("list");
}

#[cfg(unix)]
mod pty {
    use std::io::{Read, Write};
    use std::time::Duration;

    use data::serial_port::SerialLink;
    use domain::link::{Link, LinkEvent};
    use serialport::{SerialPort, TTYPort};

    use super::*;

    fn next_line(rx: &mpsc::Receiver<LinkEvent>) -> String {
        match rx.recv_timeout(Duration::from_secs(2)).expect("event") {
            LinkEvent::Line { text, .. } => text,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn lines_split_and_tail_flushes_on_timeout() {
        let (mut master, slave) = TTYPort::pair().expect("pty pair");
        let (tx, rx) = mpsc::channel();
        let _link = SerialLink::spawn(Box::new(slave), tx);
        master.write_all(b"a\r\nb").expect("write");
        assert_eq!(next_line(&rx), "a");
        assert_eq!(next_line(&rx), "b");
    }

    #[test]
    fn send_reaches_the_wire() {
        let (mut master, slave) = TTYPort::pair().expect("pty pair");
        let (tx, _rx) = mpsc::channel();
        let link = SerialLink::spawn(Box::new(slave), tx);
        link.send(b"x\r\n".to_vec()).expect("send");
        master.set_timeout(Duration::from_secs(2)).expect("timeout");
        let mut buf = [0u8; 3];
        master.read_exact(&mut buf).expect("read");
        assert_eq!(&buf, b"x\r\n");
    }

    #[test]
    fn drop_stops_the_thread() {
        let (_master, slave) = TTYPort::pair().expect("pty pair");
        let (tx, rx) = mpsc::channel();
        drop(SerialLink::spawn(Box::new(slave), tx));
        // thread joined → its Sender is gone
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        );
    }
}
