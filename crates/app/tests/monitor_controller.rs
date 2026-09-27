mod support;

use app::modules::monitor::monitor_controller::MAX_ROWS;
use app::ui::{ConnState, Tag};
use domain::AppError;
use domain::link::LinkEvent;
use slint::Model;
use support::{FakePorts, connected, line, rows, settle, setup, state};

#[test]
fn scan_selects_first_port() {
    let ports = FakePorts::with_ports(&["/dev/ttyUSB0", "/dev/ttyACM0"]);
    let (ui, _c) = setup(&ports);
    assert_eq!(state(&ui).get_port_index(), 0);
    assert_eq!(
        state(&ui).get_ports().row_data(1).expect("label"),
        "/dev/ttyACM0 — USB (Fake)"
    );
}

#[test]
fn rescan_keeps_selected_port() {
    let ports = FakePorts::with_ports(&["A", "B"]);
    let (ui, c) = setup(&ports);
    state(&ui).set_port_index(1);
    ports.ports.lock().expect("ports").reverse();
    c.scan();
    assert_eq!(state(&ui).get_port_index(), 0, "B moved to index 0");
}

#[test]
fn connect_logs_info_and_lines_become_rx_rows() {
    let (ports, ui, c) = connected(&["/dev/ttyUSB0"]);
    ports.emit(line("hello"));
    c.pump();
    assert_eq!(
        rows(&ui),
        vec![
            (
                Tag::Inf,
                "Connected to /dev/ttyUSB0 at 9600 baud".to_string()
            ),
            (Tag::Rx, "hello".to_string()),
        ]
    );
    assert_eq!(state(&ui).get_rx_count(), 1);
    assert_eq!(
        state(&ui).get_rows().row_data(1).expect("row").time,
        "12:00:00.000"
    );
}

// parity: src-tauri/tests/commands.rs + state.rs
#[test]
fn connect_twice_is_already_connected() {
    let (_ports, _ui, c) = connected(&["COM1"]);
    assert_eq!(c.connect(), Err(AppError::AlreadyConnected));
}

#[test]
fn disconnect_while_idle_is_not_connected() {
    let ports = FakePorts::with_ports(&["COM1"]);
    let (_ui, c) = setup(&ports);
    assert_eq!(c.disconnect(), Err(AppError::NotConnected));
}

#[test]
fn disconnect_logs_warning() {
    let (_ports, ui, c) = connected(&["COM1"]);
    c.disconnect().expect("disconnect");
    assert_eq!(state(&ui).get_conn_state(), ConnState::Disconnected);
    assert_eq!(
        rows(&ui).last().cloned(),
        Some((Tag::Wrn, "Disconnected from serial port".to_string()))
    );
}

#[test]
fn refused_connect_shows_banner_and_err_row() {
    let ports = FakePorts::with_ports(&["/dev/ttyACM0"]);
    *ports.refuse.lock().expect("refuse") = Some(AppError::PermissionDenied("/dev/ttyACM0".into()));
    let (ui, c) = setup(&ports);
    state(&ui).invoke_connect();
    settle(&c, &ui);
    assert_eq!(state(&ui).get_conn_state(), ConnState::Failed);
    assert_eq!(
        state(&ui).get_banner_title(),
        "Can't open /dev/ttyACM0: permission denied"
    );
    assert_eq!(
        rows(&ui),
        vec![(
            Tag::Err,
            "Connection failed: permission denied: /dev/ttyACM0".to_string()
        )]
    );
    assert_eq!(state(&ui).get_err_count(), 1);
}

#[test]
fn connect_without_port_shows_banner() {
    let ports = FakePorts::with_ports(&[]);
    let (ui, c) = setup(&ports);
    state(&ui).invoke_connect();
    assert_eq!(state(&ui).get_conn_state(), ConnState::Failed);
    assert_eq!(state(&ui).get_banner_hint(), "no port selected");
    assert_eq!(ports.connects.load(std::sync::atomic::Ordering::SeqCst), 0);
    c.pump(); // nothing pending, no panic
}

#[test]
fn retry_after_refused_connect_connects() {
    let ports = FakePorts::with_ports(&["COM3"]);
    *ports.refuse.lock().expect("refuse") = Some(AppError::PermissionDenied("COM3".into()));
    let (ui, c) = setup(&ports);
    state(&ui).invoke_connect();
    settle(&c, &ui);
    *ports.refuse.lock().expect("refuse") = None;
    state(&ui).invoke_retry();
    settle(&c, &ui);
    assert_eq!(state(&ui).get_conn_state(), ConnState::Connected);
    ports.emit(line("ok"));
    c.pump();
    assert_eq!(rows(&ui).last().cloned(), Some((Tag::Rx, "ok".to_string())));
}

#[test]
fn dismiss_banner_returns_to_disconnected() {
    let ports = FakePorts::with_ports(&[]);
    let (ui, _c) = setup(&ports);
    state(&ui).invoke_connect();
    state(&ui).invoke_dismiss_banner();
    assert_eq!(state(&ui).get_conn_state(), ConnState::Disconnected);
}

#[test]
fn link_failure_marks_failed() {
    let (ports, ui, c) = connected(&["COM1"]);
    ports.emit(LinkEvent::Failed(AppError::Port("device unplugged".into())));
    ports.emit(line("after death"));
    c.pump();
    assert_eq!(state(&ui).get_conn_state(), ConnState::Failed);
    assert_eq!(
        rows(&ui).last().cloned(),
        Some((Tag::Err, "Link failed: port: device unplugged".to_string()))
    );
    assert_eq!(c.disconnect(), Err(AppError::NotConnected), "link dropped");
}

#[test]
fn filter_drops_line() {
    let (ports, ui, c) = connected(&["COM1"]);
    state(&ui).set_filter_on(true);
    state(&ui).set_skip("10".into());
    ports.emit(line("abc"));
    c.pump();
    assert_eq!(state(&ui).get_rx_count(), 0);
    assert!(rows(&ui).iter().all(|(tag, _)| *tag != Tag::Rx));
}

#[test]
fn frame_change_updates_label() {
    let ports = FakePorts::with_ports(&["COM1"]);
    let (ui, _c) = setup(&ports);
    state(&ui).set_parity_index(1);
    state(&ui).invoke_frame_changed();
    assert_eq!(state(&ui).get_frame_label(), "8E1 · no flow");
}

#[test]
fn about_shows_version_and_author() {
    let ports = FakePorts::with_ports(&[]);
    let (ui, _c) = setup(&ports);
    assert_eq!(state(&ui).get_app_version(), "0.4.0");
    assert_eq!(state(&ui).get_app_author(), "kuhlen");
}

#[test]
fn send_writes_bytes_and_logs_tx() {
    let (ports, ui, _c) = connected(&["COM1"]);
    state(&ui).set_send_text("AT".into());
    state(&ui).invoke_send("AT".into(), 3);
    assert_eq!(*ports.sent.lock().expect("sent"), vec![b"AT\r\n".to_vec()]);
    assert_eq!(rows(&ui).last().cloned(), Some((Tag::Tx, "AT".to_string())));
    assert_eq!(state(&ui).get_tx_count(), 1);
    assert_eq!(state(&ui).get_send_text(), "", "input cleared after send");
}

#[test]
fn empty_send_without_ending_is_ignored() {
    let (ports, ui, c) = connected(&["COM1"]);
    c.send("", 0).expect("no-op");
    assert!(ports.sent.lock().expect("sent").is_empty());
    assert_eq!(state(&ui).get_tx_count(), 0);
}

#[test]
fn send_while_disconnected_is_not_connected() {
    let ports = FakePorts::with_ports(&["COM1"]);
    let (_ui, c) = setup(&ports);
    assert_eq!(c.send("AT", 3), Err(AppError::NotConnected));
}

// banner follows when the link thread's Failed event is pumped
#[test]
fn send_on_dead_link_logs_error_row() {
    let (ports, ui, _c) = connected(&["COM1"]);
    ports.dead.store(true, std::sync::atomic::Ordering::SeqCst);
    state(&ui).invoke_send("AT".into(), 3);
    assert_eq!(
        rows(&ui).last().cloned(),
        Some((Tag::Err, "Send failed: not connected".to_string()))
    );
}

#[test]
fn clear_empties_rows_and_counters() {
    let (ports, ui, c) = connected(&["COM1"]);
    ports.emit(line("x"));
    c.pump();
    state(&ui).invoke_clear();
    assert!(rows(&ui).is_empty());
    assert_eq!(
        (
            state(&ui).get_rx_count(),
            state(&ui).get_tx_count(),
            state(&ui).get_err_count()
        ),
        (0, 0, 0)
    );
}

#[test]
fn preview_follows_last_raw_line() {
    let (ports, ui, c) = connected(&["COM1"]);
    state(&ui).set_filter_on(true);
    state(&ui).set_skip("2".into());
    state(&ui).set_keep("3".into());
    ports.emit(line("ABCDEFG"));
    c.pump();
    assert_eq!(
        rows(&ui).last().cloned(),
        Some((Tag::Rx, "CDE".to_string()))
    );
    let s = state(&ui);
    assert_eq!(
        (
            s.get_preview_skipped(),
            s.get_preview_kept(),
            s.get_preview_cut()
        ),
        ("AB".into(), "CDE".into(), "FG".into())
    );
    // editing the filter re-previews without new data
    s.set_keep(String::new().into());
    s.invoke_filter_changed();
    assert_eq!(s.get_preview_kept(), "CDEFG");
}

#[test]
fn rows_capped_oldest_dropped() {
    let (ports, ui, c) = connected(&["COM1"]);
    for i in 0..MAX_ROWS + 5 {
        ports.emit(line(&i.to_string()));
    }
    c.pump();
    let all = rows(&ui);
    assert_eq!(all.len(), MAX_ROWS);
    // INF row + lines 0..=4 dropped
    assert_eq!(all[0], (Tag::Rx, "5".to_string()));
    assert_eq!(
        state(&ui).get_rx_count(),
        i32::try_from(MAX_ROWS + 5).expect("fits")
    );
}
