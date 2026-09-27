//! fake SerialPorts + helpers; call setup() first in every test
#![allow(dead_code)] // each test binary uses a subset

use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use app::modules::monitor::monitor_controller::MonitorController;
use app::ui::{AppWindow, ConnState, MonitorState, Tag};
use domain::AppError;
use domain::link::{Link, LinkEvent, SerialPorts};
use domain::serial::{PortInfo, SerialConfig};
use slint::{ComponentHandle, Model};

#[derive(Default)]
pub struct FakePorts {
    pub ports: Mutex<Vec<PortInfo>>,
    pub refuse: Mutex<Option<AppError>>,
    pub events: Mutex<Option<Sender<LinkEvent>>>,
    pub sent: Arc<Mutex<Vec<Vec<u8>>>>,
    pub dead: Arc<AtomicBool>,
    pub connects: AtomicUsize,
}

impl FakePorts {
    pub fn with_ports(names: &[&str]) -> Arc<Self> {
        let fake = Self::default();
        *fake.ports.lock().expect("ports") = names
            .iter()
            .map(|n| PortInfo {
                name: (*n).into(),
                port_type: "USB (Fake)".into(),
            })
            .collect();
        Arc::new(fake)
    }

    pub fn emit(&self, event: LinkEvent) {
        let events = self.events.lock().expect("events");
        events
            .as_ref()
            .expect("connected")
            .send(event)
            .expect("receiver alive");
    }
}

impl SerialPorts for FakePorts {
    fn list(&self) -> Result<Vec<PortInfo>, AppError> {
        Ok(self.ports.lock().expect("ports").clone())
    }

    fn connect(
        &self,
        _config: &SerialConfig,
        events: Sender<LinkEvent>,
    ) -> Result<Box<dyn Link>, AppError> {
        self.connects.fetch_add(1, Ordering::SeqCst);
        if let Some(e) = self.refuse.lock().expect("refuse").clone() {
            return Err(e);
        }
        *self.events.lock().expect("events") = Some(events);
        Ok(Box::new(FakeLink {
            sent: Arc::clone(&self.sent),
            dead: Arc::clone(&self.dead),
        }))
    }
}

struct FakeLink {
    sent: Arc<Mutex<Vec<Vec<u8>>>>,
    dead: Arc<AtomicBool>,
}

impl Link for FakeLink {
    fn send(&self, bytes: Vec<u8>) -> Result<(), AppError> {
        if self.dead.load(Ordering::SeqCst) {
            return Err(AppError::NotConnected);
        }
        self.sent.lock().expect("sent").push(bytes);
        Ok(())
    }
}

pub fn setup(ports: &Arc<FakePorts>) -> (AppWindow, Rc<MonitorController>) {
    i_slint_backend_testing::init_no_event_loop();
    let ui = AppWindow::new().expect("window");
    let controller = MonitorController::new(ports.clone(), &ui);
    (ui, controller)
}

pub fn state(ui: &AppWindow) -> MonitorState<'_> {
    ui.global::<MonitorState>()
}

/// connect runs on a thread: pump until it lands
pub fn settle(controller: &MonitorController, ui: &AppWindow) {
    for _ in 0..400 {
        controller.pump();
        if state(ui).get_conn_state() != ConnState::Connecting {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("connect never finished");
}

pub fn connected(names: &[&str]) -> (Arc<FakePorts>, AppWindow, Rc<MonitorController>) {
    let ports = FakePorts::with_ports(names);
    let (ui, controller) = setup(&ports);
    controller.connect().expect("connect");
    settle(&controller, &ui);
    assert_eq!(state(&ui).get_conn_state(), ConnState::Connected);
    (ports, ui, controller)
}

pub fn line(text: &str) -> LinkEvent {
    LinkEvent::Line {
        text: text.into(),
        at: "12:00:00.000".into(),
    }
}

pub fn rows(ui: &AppWindow) -> Vec<(Tag, String)> {
    state(ui)
        .get_rows()
        .iter()
        .map(|r| (r.tag, r.text.to_string()))
        .collect()
}
