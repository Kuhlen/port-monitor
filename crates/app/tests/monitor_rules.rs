use app::modules::monitor::monitor_rules::{
    Frame, banner_text, config_from_form, filter_from_form, pick_port, port_label,
};
use domain::AppError;
use domain::serial::{BaudRate, DataBits, FlowControl, Parity, PortInfo, SerialConfig, StopBits};

fn port(name: &str) -> PortInfo {
    PortInfo {
        name: name.into(),
        port_type: "USB (CH340)".into(),
    }
}

#[test]
fn default_frame_label() {
    assert_eq!(Frame::default().label(), "8N1 · no flow");
}

#[test]
fn frame_label_from_indices() {
    assert_eq!(Frame::from_indices(0, 2, 1, 1).label(), "5O2 · RTS/CTS");
    assert_eq!(Frame::from_indices(2, 1, 0, 2).label(), "7E1 · XON/XOFF");
}

#[test]
fn out_of_range_index_falls_back_to_default() {
    assert_eq!(Frame::from_indices(9, -1, 5, 7), Frame::default());
}

#[test]
fn form_builds_config() {
    let frame = Frame::from_indices(3, 0, 0, 0);
    let config = config_from_form(&[port("/dev/ttyUSB0")], 0, "115200", frame).expect("valid");
    assert_eq!(
        config,
        SerialConfig {
            port: "/dev/ttyUSB0".into(),
            baud_rate: BaudRate::parse("115200").expect("baud"),
            data_bits: DataBits::Eight,
            parity: Parity::None,
            stop_bits: StopBits::One,
            flow: FlowControl::None,
        }
    );
}

#[test]
fn form_without_port_is_error() {
    let result = config_from_form(&[], -1, "9600", Frame::default());
    assert_eq!(result, Err(AppError::Port("no port selected".into())));
}

#[test]
fn form_with_bad_baud_is_error() {
    for baud in ["0", "abc", ""] {
        let result = config_from_form(&[port("COM3")], 0, baud, Frame::default());
        assert!(matches!(result, Err(AppError::Port(_))), "{baud:?}");
    }
}

#[test]
fn filter_form_parses_numbers() {
    let f = filter_from_form(true, "2", "3", ";");
    assert_eq!(
        (f.enabled, f.offset, f.length, f.exclude.as_str()),
        (true, 2, Some(3), ";")
    );
}

// old form: parse().unwrap_or(0) / parse().ok()
#[test]
fn filter_form_tolerates_garbage() {
    let f = filter_from_form(true, "abc", "", "");
    assert_eq!((f.offset, f.length), (0, None));
    let f = filter_from_form(true, " 2", "x", "");
    assert_eq!((f.offset, f.length), (2, None));
}

#[test]
fn port_label_shows_type() {
    assert_eq!(port_label(&port("COM3")), "COM3 — USB (CH340)");
}

#[test]
fn pick_port_keeps_selection_else_first() {
    let ports = [port("A"), port("B")];
    assert_eq!(pick_port(&ports, Some("B")), 1);
    assert_eq!(pick_port(&ports, Some("gone")), 0);
    assert_eq!(pick_port(&ports, None), 0);
    assert_eq!(pick_port(&[], None), -1);
}

#[test]
fn permission_banner_names_port() {
    let (title, hint) = banner_text(&AppError::PermissionDenied("/dev/ttyACM0".into()));
    assert_eq!(title, "Can't open /dev/ttyACM0: permission denied");
    if cfg!(target_os = "linux") {
        assert!(hint.contains("dialout"), "{hint}");
    } else if cfg!(windows) {
        assert!(hint.contains("in use by another program"), "{hint}");
    }
}

#[test]
fn port_error_banner_carries_reason() {
    let (title, hint) = banner_text(&AppError::Port("failed to open COM9: gone".into()));
    assert_eq!(title, "Serial port error");
    assert_eq!(hint, "failed to open COM9: gone");
}
