use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use port_monitor_core::features::serial::{PortInfo, SerialApi, SerialConfig};
use port_monitor_core::features::update::{UpdateApi, UpdateCheck};
use port_monitor_core::AppError;
use tauri::{AppHandle, Runtime};

use crate::infra;

struct Reader {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

pub struct AppState<R: Runtime> {
    app: AppHandle<R>,
    reader: Mutex<Option<Reader>>,
    // Update from check is held until the user hits install. Cannot re-check:
    // download_and_install consumes the object.
    #[cfg(desktop)]
    pending: Mutex<Option<tauri_plugin_updater::Update>>,
}

impl<R: Runtime> AppState<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self {
            app,
            reader: Mutex::new(None),
            #[cfg(desktop)]
            pending: Mutex::new(None),
        }
    }
}

fn poisoned(e: impl std::fmt::Display) -> AppError {
    AppError::Port(e.to_string())
}

impl<R: Runtime> SerialApi for AppState<R> {
    async fn list_ports(&self) -> Result<Vec<PortInfo>, AppError> {
        infra::available_ports()
    }

    async fn connect_port(&self, config: SerialConfig) -> Result<(), AppError> {
        // The real gate. Frontend calls validate() too, but only for instant
        // feedback.
        let settings = config.validate()?;

        let mut reader = self.reader.lock().map_err(poisoned)?;
        if reader.is_some() {
            return Err(AppError::AlreadyConnected);
        }

        let port = infra::open(&config, &settings)?;
        let stop = Arc::new(AtomicBool::new(false));
        let handle = infra::spawn_reader(self.app.clone(), port, stop.clone());

        *reader = Some(Reader {
            stop,
            handle: Some(handle),
        });
        Ok(())
    }

    async fn disconnect_port(&self) -> Result<(), AppError> {
        let taken = self.reader.lock().map_err(poisoned)?.take();
        let Some(reader) = taken else {
            return Err(AppError::NotConnected);
        };

        reader.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = reader.handle {
            let _ = handle.join();
        }
        Ok(())
    }
}

#[cfg(desktop)]
impl<R: Runtime> UpdateApi for AppState<R> {
    async fn check_update(&self) -> Result<UpdateCheck, AppError> {
        let Some(update) = infra::check_release(&self.app).await? else {
            return Ok(UpdateCheck::UpToDate);
        };
        let version = update.version.clone();
        *self.pending.lock().map_err(poisoned)? = Some(update);
        Ok(UpdateCheck::Available { version })
    }

    async fn install_update(&self) -> Result<(), AppError> {
        // Take it out of the Mutex first; a guard must not live across await.
        let taken = self.pending.lock().map_err(poisoned)?.take();
        let Some(update) = taken else {
            return Err(AppError::Update("no pending update".into()));
        };
        infra::install_release(update).await
    }
}

// Mobile has no tauri-plugin-updater. Commands stay registered so the
// frontend has one call shape on every platform.
#[cfg(not(desktop))]
impl<R: Runtime> UpdateApi for AppState<R> {
    async fn check_update(&self) -> Result<UpdateCheck, AppError> {
        Ok(UpdateCheck::UpToDate)
    }

    async fn install_update(&self) -> Result<(), AppError> {
        Err(AppError::Update("updater is desktop-only".into()))
    }
}
