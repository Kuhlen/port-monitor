use port_monitor_core::features::serial::{
    PortInfo, SerialApi, SerialConfig, SerialDataEvent, SerialErrorEvent, EVENT_SERIAL_DATA,
    EVENT_SERIAL_ERROR,
};
use port_monitor_core::features::update::{UpdateApi, UpdateCheck};
use port_monitor_core::AppError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], js_name = "listen")]
    async fn listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>) -> JsValue;
}

pub fn tauri_available() -> bool {
    let Some(win) = web_sys::window() else {
        return false;
    };
    js_sys::Reflect::get(&win, &JsValue::from_str("__TAURI__"))
        .ok()
        .map(|v| !v.is_undefined() && !v.is_null())
        .unwrap_or(false)
}

// No Tauri, no call. Caller MUST treat IpcUnavailable as a silent no-op, not
// a screen error - the frontend has to stay alive in a plain browser.
async fn call<A: Serialize, T: DeserializeOwned>(cmd: &str, args: A) -> Result<T, AppError> {
    if !tauri_available() {
        return Err(AppError::IpcUnavailable);
    }
    let jsv =
        serde_wasm_bindgen::to_value(&args).map_err(|e| AppError::Validation(e.to_string()))?;
    let v = invoke(cmd, jsv).await.map_err(decode_err)?;
    serde_wasm_bindgen::from_value::<T>(v).map_err(|e| AppError::Port(e.to_string()))
}

// Backend returns a serialized AppError. Restore it so the frontend matches
// on the enum instead of sniffing strings.
fn decode_err(e: JsValue) -> AppError {
    serde_wasm_bindgen::from_value::<AppError>(e.clone())
        .unwrap_or_else(|_| AppError::Port(e.as_string().unwrap_or_else(|| format!("{e:?}"))))
}

#[derive(Clone, Copy)]
pub struct Bridge;

// Typed structs, never serde_json::json!: serde_wasm_bindgen turns maps into
// a JS Map and Tauri rejects that.
#[derive(Serialize)]
struct NoArgs {}

#[derive(Serialize)]
struct ConfigArgs {
    config: SerialConfig,
}

impl SerialApi for Bridge {
    async fn list_ports(&self) -> Result<Vec<PortInfo>, AppError> {
        call("list_ports", NoArgs {}).await
    }

    async fn connect_port(&self, config: SerialConfig) -> Result<(), AppError> {
        // Instant feedback. Real gate stays in the backend.
        config.validate()?;
        call("connect_port", ConfigArgs { config }).await
    }

    async fn disconnect_port(&self) -> Result<(), AppError> {
        call("disconnect_port", NoArgs {}).await
    }
}

impl UpdateApi for Bridge {
    async fn check_update(&self) -> Result<UpdateCheck, AppError> {
        call("check_update", NoArgs {}).await
    }

    async fn install_update(&self) -> Result<(), AppError> {
        call("install_update", NoArgs {}).await
    }
}

#[derive(Deserialize)]
struct TauriEvent<T> {
    payload: T,
}

pub fn on_serial_data(callback: impl FnMut(SerialDataEvent) + 'static) {
    subscribe(EVENT_SERIAL_DATA, callback);
}

pub fn on_serial_error(callback: impl FnMut(SerialErrorEvent) + 'static) {
    subscribe(EVENT_SERIAL_ERROR, callback);
}

fn subscribe<T: DeserializeOwned + 'static>(
    event: &'static str,
    mut callback: impl FnMut(T) + 'static,
) {
    if !tauri_available() {
        return;
    }
    let closure = Closure::new(move |raw: JsValue| {
        if let Ok(evt) = serde_wasm_bindgen::from_value::<TauriEvent<T>>(raw) {
            callback(evt.payload);
        }
    });

    wasm_bindgen_futures::spawn_local(async move {
        listen(event, &closure).await;
        closure.forget();
    });
}
