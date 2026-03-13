use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub struct SerialConnection {
    pub stop_flag: Arc<AtomicBool>,
    pub thread_handle: Option<std::thread::JoinHandle<()>>,
}

pub type AppState = Mutex<Option<SerialConnection>>;
