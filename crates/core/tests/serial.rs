use port_monitor_core::features::serial::{DataBits, Parity, SerialConfig};
use port_monitor_core::AppError;

fn cfg() -> SerialConfig {
    SerialConfig {
        port: "/dev/ttyUSB0".into(),
        baud_rate: 9600,
        data_bits: "8".into(),
        stop_bits: "1".into(),
        parity: "none".into(),
        flow_control: "none".into(),
    }
}

#[test]
fn valid_config_yields_settings() {
    let s = cfg().validate().expect("config is valid");
    assert_eq!(s.data_bits, DataBits::Eight);
    assert_eq!(s.parity, Parity::None);
}

#[test]
fn broken_config_rejected() {
    for broken in [
        SerialConfig {
            port: "  ".into(),
            ..cfg()
        },
        SerialConfig {
            baud_rate: 0,
            ..cfg()
        },
        SerialConfig {
            data_bits: "9".into(),
            ..cfg()
        },
        SerialConfig {
            stop_bits: "3".into(),
            ..cfg()
        },
        SerialConfig {
            parity: "maybe".into(),
            ..cfg()
        },
        SerialConfig {
            flow_control: "magic".into(),
            ..cfg()
        },
    ] {
        assert!(
            matches!(broken.validate(), Err(AppError::Validation(_))),
            "must be rejected: {broken:?}"
        );
    }
}
