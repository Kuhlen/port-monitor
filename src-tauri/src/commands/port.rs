use crate::types::PortInfo;

#[tauri::command]
pub fn list_ports() -> Result<Vec<PortInfo>, String> {
    let ports = serialport::available_ports().map_err(|e| e.to_string())?;

    let port_list = ports
        .into_iter()
        .map(|p| {
            let port_type = match &p.port_type {
                serialport::SerialPortType::UsbPort(info) => {
                    format!("USB ({})", info.product.as_deref().unwrap_or("Unknown"))
                }
                serialport::SerialPortType::BluetoothPort => "Bluetooth".to_string(),
                serialport::SerialPortType::PciPort => "PCI".to_string(),
                serialport::SerialPortType::Unknown => "Unknown".to_string(),
            };
            PortInfo {
                name: p.port_name,
                port_type,
            }
        })
        .collect();

    Ok(port_list)
}
