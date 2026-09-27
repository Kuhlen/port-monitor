//! Monitor page: link lifecycle, console rows, filter preview.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};

use domain::AppError;
use domain::filter::LineFilter;
use domain::link::{Link, LinkEvent, SerialPorts};
use domain::send::{LineEnding, encode};
use domain::serial::{PortInfo, SerialConfig};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use super::monitor_rules::{self, Frame};
use crate::ui::{AppWindow, ConnState, ConsoleRow, MonitorState, Tag};

// ponytail: capped, oldest dropped (old app unbounded); VecModel::remove(0) is O(n), ring model if it shows
pub const MAX_ROWS: usize = 10_000;

type Opened = Result<Box<dyn Link>, AppError>;

// events receiver lives per connection: a dead link's queued events die with it
enum Conn {
    Idle,
    Opening {
        config: SerialConfig,
        result: Receiver<Opened>,
        events: Receiver<LinkEvent>,
    },
    Open {
        link: Box<dyn Link>,
        events: Receiver<LinkEvent>,
    },
}

pub struct MonitorController {
    ports: Arc<dyn SerialPorts>,
    ui: slint::Weak<AppWindow>,
    found: RefCell<Vec<PortInfo>>,
    conn: RefCell<Conn>,
    rows: Rc<VecModel<ConsoleRow>>,
    last_line: RefCell<Option<String>>,
}

impl MonitorController {
    pub fn new(ports: Arc<dyn SerialPorts>, ui: &AppWindow) -> Rc<Self> {
        let this = Rc::new(Self {
            ports,
            ui: ui.as_weak(),
            found: RefCell::default(),
            conn: RefCell::new(Conn::Idle),
            rows: Rc::new(VecModel::default()),
            last_line: RefCell::default(),
        });
        // globals outlive pages: reset what Rust owns
        let state = ui.global::<MonitorState>();
        state.set_conn_state(ConnState::Disconnected);
        state.set_ports(ModelRc::default());
        state.set_rows(ModelRc::from(this.rows.clone()));
        state.set_rx_count(0);
        state.set_tx_count(0);
        state.set_err_count(0);
        state.set_banner_title(SharedString::default());
        state.set_banner_hint(SharedString::default());
        state.set_app_version(env!("CARGO_PKG_VERSION").into());
        state.set_app_author(env!("CARGO_PKG_AUTHORS").into());
        this.wire(&state);
        this.frame_changed();
        this.refresh_preview();
        this.scan();
        this
    }

    fn wire(self: &Rc<Self>, state: &MonitorState) {
        let weak = Rc::downgrade(self);
        let on = |f: fn(&Self)| {
            let weak = weak.clone();
            move || {
                if let Some(c) = weak.upgrade() {
                    f(&c);
                }
            }
        };
        state.on_scan(on(Self::scan));
        state.on_connect(on(|c| c.report("Connection failed", c.connect())));
        state.on_disconnect(on(|c| c.report("Disconnect failed", c.disconnect())));
        state.on_dismiss_banner(on(Self::dismiss_banner));
        state.on_filter_changed(on(Self::refresh_preview));
        state.on_frame_changed(on(Self::frame_changed));
        state.on_clear(on(Self::clear));
        state.on_send({
            let weak = weak.clone();
            move |text, ending| {
                if let Some(c) = weak.upgrade() {
                    c.report("Send failed", c.send(&text, ending));
                }
            }
        });
    }

    fn ui(&self) -> AppWindow {
        self.ui.upgrade().expect("window outlives controller")
    }

    pub fn scan(&self) {
        match self.ports.list() {
            Ok(found) => {
                let ui = self.ui();
                let state = ui.global::<MonitorState>();
                let current = usize::try_from(state.get_port_index())
                    .ok()
                    .and_then(|i| self.found.borrow().get(i).map(|p| p.name.clone()));
                let index = monitor_rules::pick_port(&found, current.as_deref());
                let labels: Vec<SharedString> = found
                    .iter()
                    .map(|p| monitor_rules::port_label(p).into())
                    .collect();
                state.set_ports(ModelRc::new(VecModel::from(labels)));
                state.set_port_index(index);
                *self.found.borrow_mut() = found;
            }
            Err(e) => self.push(Tag::Err, &format!("Scan failed: {e}")),
        }
    }

    pub fn connect(&self) -> Result<(), AppError> {
        if !matches!(*self.conn.borrow(), Conn::Idle) {
            return Err(AppError::AlreadyConnected);
        }
        let config = self.read_config()?;
        let (event_tx, events) = mpsc::channel();
        let (result_tx, result) = mpsc::channel();
        let ports = Arc::clone(&self.ports);
        let job = config.clone();
        // open can block for seconds (Windows Bluetooth COM ports)
        std::thread::spawn(move || {
            let _ = result_tx.send(ports.connect(&job, event_tx));
        });
        *self.conn.borrow_mut() = Conn::Opening {
            config,
            result,
            events,
        };
        self.ui()
            .global::<MonitorState>()
            .set_conn_state(ConnState::Connecting);
        Ok(())
    }

    pub fn disconnect(&self) -> Result<(), AppError> {
        if !matches!(*self.conn.borrow(), Conn::Open { .. }) {
            return Err(AppError::NotConnected);
        }
        // drop joins the link thread
        *self.conn.borrow_mut() = Conn::Idle;
        self.ui()
            .global::<MonitorState>()
            .set_conn_state(ConnState::Disconnected);
        self.push(Tag::Wrn, "Disconnected from serial port");
        Ok(())
    }

    fn dismiss_banner(&self) {
        let ui = self.ui();
        let state = ui.global::<MonitorState>();
        if state.get_conn_state() == ConnState::Failed {
            state.set_conn_state(ConnState::Disconnected);
        }
    }

    /// called every pump tick (and by tests)
    pub fn pump(&self) {
        let opened = match &*self.conn.borrow() {
            Conn::Opening { result, .. } => match result.try_recv() {
                Ok(opened) => Some(opened),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => {
                    Some(Err(AppError::Port("connect thread died".into())))
                }
            },
            _ => None,
        };
        if let Some(opened) = opened {
            self.finish_open(opened);
        }
        let batch: Vec<LinkEvent> = match &*self.conn.borrow() {
            Conn::Open { events, .. } => events.try_iter().collect(),
            _ => Vec::new(),
        };
        self.on_events(batch);
    }

    fn finish_open(&self, opened: Opened) {
        let Conn::Opening { config, events, .. } =
            std::mem::replace(&mut *self.conn.borrow_mut(), Conn::Idle)
        else {
            return;
        };
        match opened {
            Ok(link) => {
                *self.conn.borrow_mut() = Conn::Open { link, events };
                self.ui()
                    .global::<MonitorState>()
                    .set_conn_state(ConnState::Connected);
                self.push(
                    Tag::Inf,
                    &format!("Connected to {} at {} baud", config.port, config.baud_rate),
                );
            }
            Err(e) => self.fail("Connection failed", &e),
        }
    }

    pub fn on_events(&self, batch: Vec<LinkEvent>) {
        if batch.is_empty() {
            return;
        }
        let filter = self.read_filter();
        for event in batch {
            match event {
                LinkEvent::Line { text, at } => {
                    if let Some(shown) = filter.apply(&text) {
                        self.push_at(at, Tag::Rx, &shown);
                    }
                    *self.last_line.borrow_mut() = Some(text);
                }
                LinkEvent::Failed(e) => {
                    // rest of the batch belongs to the dead link; Failed can also come from a write error
                    self.fail("Link failed", &e);
                    break;
                }
            }
        }
        self.refresh_preview();
    }

    pub fn send(&self, text: &str, ending: i32) -> Result<(), AppError> {
        let bytes = encode(text, monitor_rules::pick(&LineEnding::ALL, ending));
        match &*self.conn.borrow() {
            Conn::Open { .. } if bytes.is_empty() => return Ok(()),
            Conn::Open { link, .. } => link.send(bytes)?,
            _ => return Err(AppError::NotConnected),
        }
        self.push(Tag::Tx, text);
        self.ui()
            .global::<MonitorState>()
            .set_send_text(SharedString::default());
        Ok(())
    }

    pub fn clear(&self) {
        self.rows.set_vec(Vec::new());
        let ui = self.ui();
        let state = ui.global::<MonitorState>();
        state.set_rx_count(0);
        state.set_tx_count(0);
        state.set_err_count(0);
    }

    fn refresh_preview(&self) {
        let filter = self.read_filter();
        let preview = self
            .last_line
            .borrow()
            .as_deref()
            .map(|line| filter.preview(line))
            .unwrap_or_default();
        let ui = self.ui();
        let state = ui.global::<MonitorState>();
        state.set_preview_skipped(preview.skipped.into());
        state.set_preview_kept(preview.kept.into());
        state.set_preview_cut(preview.cut.into());
    }

    fn frame_changed(&self) {
        let ui = self.ui();
        let state = ui.global::<MonitorState>();
        state.set_frame_label(self.read_frame(&state).label().into());
    }

    /// AlreadyConnected / NotConnected: live link (or none) stays, row only
    fn report(&self, prefix: &str, result: Result<(), AppError>) {
        match result {
            Ok(()) => {}
            Err(e @ (AppError::AlreadyConnected | AppError::NotConnected)) => {
                self.push(Tag::Err, &format!("{prefix}: {e}"));
            }
            Err(e) => self.fail(prefix, &e),
        }
    }

    fn fail(&self, prefix: &str, error: &AppError) {
        *self.conn.borrow_mut() = Conn::Idle;
        let (title, hint) = monitor_rules::banner_text(error);
        let ui = self.ui();
        let state = ui.global::<MonitorState>();
        state.set_banner_title(title.into());
        state.set_banner_hint(hint.into());
        state.set_conn_state(ConnState::Failed);
        self.push(Tag::Err, &format!("{prefix}: {error}"));
    }

    fn read_frame(&self, state: &MonitorState) -> Frame {
        Frame::from_indices(
            state.get_data_bits_index(),
            state.get_parity_index(),
            state.get_stop_bits_index(),
            state.get_flow_index(),
        )
    }

    fn read_config(&self) -> Result<SerialConfig, AppError> {
        let ui = self.ui();
        let state = ui.global::<MonitorState>();
        monitor_rules::config_from_form(
            &self.found.borrow(),
            state.get_port_index(),
            &state.get_baud(),
            self.read_frame(&state),
        )
    }

    fn read_filter(&self) -> LineFilter {
        let ui = self.ui();
        let state = ui.global::<MonitorState>();
        monitor_rules::filter_from_form(
            state.get_filter_on(),
            &state.get_skip(),
            &state.get_keep(),
            &state.get_remove(),
        )
    }

    fn push(&self, tag: Tag, text: &str) {
        self.push_at(now(), tag, text);
    }

    fn push_at(&self, time: String, tag: Tag, text: &str) {
        if self.rows.row_count() >= MAX_ROWS {
            self.rows.remove(0);
        }
        self.rows.push(ConsoleRow {
            time: time.into(),
            tag,
            text: text.into(),
        });
        let ui = self.ui();
        let state = ui.global::<MonitorState>();
        match tag {
            Tag::Rx => state.set_rx_count(state.get_rx_count() + 1),
            Tag::Tx => state.set_tx_count(state.get_tx_count() + 1),
            Tag::Err => state.set_err_count(state.get_err_count() + 1),
            Tag::Inf | Tag::Wrn => {}
        }
    }
}

fn now() -> String {
    chrono::Local::now()
        .format(domain::link::TIME_FORMAT)
        .to_string()
}
