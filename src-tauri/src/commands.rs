use port_monitor_core::features::serial::{PortInfo, SerialApi, SerialConfig};
use port_monitor_core::features::update::{UpdateApi, UpdateCheck};
use port_monitor_core::AppError;
use tauri::{AppHandle, Manager, Runtime};

use crate::state::AppState;

type Res<T> = Result<T, AppError>;

// Pure delegation. AppHandle<R> sits in every signature so Tauri can infer
// R; State alone is not enough.
#[tauri::command]
pub async fn list_ports<R: Runtime>(app: AppHandle<R>) -> Res<Vec<PortInfo>> {
    app.state::<AppState<R>>().list_ports().await
}

#[tauri::command]
pub async fn connect_port<R: Runtime>(app: AppHandle<R>, config: SerialConfig) -> Res<()> {
    app.state::<AppState<R>>().connect_port(config).await
}

#[tauri::command]
pub async fn disconnect_port<R: Runtime>(app: AppHandle<R>) -> Res<()> {
    app.state::<AppState<R>>().disconnect_port().await
}

#[tauri::command]
pub async fn check_update<R: Runtime>(app: AppHandle<R>) -> Res<UpdateCheck> {
    app.state::<AppState<R>>().check_update().await
}

#[tauri::command]
pub async fn install_update<R: Runtime>(app: AppHandle<R>) -> Res<()> {
    app.state::<AppState<R>>().install_update().await
}
