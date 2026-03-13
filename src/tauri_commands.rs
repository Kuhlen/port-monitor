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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub version: String,
}

pub async fn check_for_update() -> Result<Option<UpdateInfo>, String> {
    let check_fn = js_sys::Reflect::get(
        &js_sys::Reflect::get(&js_sys::global(), &"__TAURI__".into())
            .map_err(|_| "Tauri not found".to_string())?,
        &"updater".into(),
    )
    .and_then(|updater| js_sys::Reflect::get(&updater, &"check".into()))
    .map_err(|_| "Updater plugin not found".to_string())?;

    let check_fn: js_sys::Function = check_fn.into();
    let promise = check_fn
        .call0(&JsValue::NULL)
        .map_err(|e| format!("{e:?}"))?;
    let result = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(|e| format!("{e:?}"))?;

    if result.is_null() || result.is_undefined() {
        return Ok(None);
    }

    let available = js_sys::Reflect::get(&result, &"available".into())
        .unwrap_or(JsValue::FALSE)
        .as_bool()
        .unwrap_or(false);

    let version = js_sys::Reflect::get(&result, &"version".into())
        .unwrap_or(JsValue::from_str("unknown"))
        .as_string()
        .unwrap_or_else(|| "unknown".to_string());

    if available {
        // Store the update object globally so we can call downloadAndInstall later
        let _ = js_sys::Reflect::set(&js_sys::global(), &"__PENDING_UPDATE__".into(), &result);
        Ok(Some(UpdateInfo { available, version }))
    } else {
        Ok(None)
    }
}

pub async fn install_update() -> Result<(), String> {
    let update = js_sys::Reflect::get(&js_sys::global(), &"__PENDING_UPDATE__".into())
        .map_err(|_| "No pending update".to_string())?;

    let install_fn = js_sys::Reflect::get(&update, &"downloadAndInstall".into())
        .map_err(|_| "downloadAndInstall not found".to_string())?;

    let install_fn: js_sys::Function = install_fn.into();
    let promise = install_fn.call0(&update).map_err(|e| format!("{e:?}"))?;
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(|e| format!("{e:?}"))?;

    Ok(())
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
