use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = "invoke")]
    async fn tauri_invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], js_name = "listen")]
    async fn tauri_listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>) -> JsValue;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub name: String,
    pub port_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: String,
    pub stop_bits: String,
    pub parity: String,
    pub flow_control: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialDataEvent {
    pub data: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialErrorEvent {
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TauriEvent<T> {
    pub payload: T,
}

pub async fn list_ports() -> Result<Vec<PortInfo>, String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).map_err(|e| e.to_string())?;
    let result = tauri_invoke("list_ports", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn connect_port(config: &SerialConfig) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "config": config }))
        .map_err(|e| e.to_string())?;
    let result = tauri_invoke("connect_port", args).await;

    if result.is_null() || result.is_undefined() {
        Ok(())
    } else {
        let err: Result<(), String> =
            serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())?;
        err
    }
}

pub async fn disconnect_port() -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).map_err(|e| e.to_string())?;
    let result = tauri_invoke("disconnect_port", args).await;

    if result.is_null() || result.is_undefined() {
        Ok(())
    } else {
        let err: Result<(), String> =
            serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())?;
        err
    }
}

pub fn listen_serial_data(callback: impl FnMut(SerialDataEvent) + 'static) {
    let mut callback = callback;
    let closure = Closure::new(move |event: JsValue| {
        if let Ok(evt) = serde_wasm_bindgen::from_value::<TauriEvent<SerialDataEvent>>(event) {
            callback(evt.payload);
        }
    });

    wasm_bindgen_futures::spawn_local(async move {
        tauri_listen("serial-data", &closure).await;
        closure.forget();
    });
}

pub fn listen_serial_error(callback: impl FnMut(SerialErrorEvent) + 'static) {
    let mut callback = callback;
    let closure = Closure::new(move |event: JsValue| {
        if let Ok(evt) = serde_wasm_bindgen::from_value::<TauriEvent<SerialErrorEvent>>(event) {
            callback(evt.payload);
        }
    });

    wasm_bindgen_futures::spawn_local(async move {
        tauri_listen("serial-error", &closure).await;
        closure.forget();
    });
}
