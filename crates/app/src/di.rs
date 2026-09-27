//! composition root

use std::rc::Rc;
use std::sync::Arc;

use data::serial_port::SerialPortLink;

use crate::modules::monitor::monitor_controller::MonitorController;
use crate::ui::AppWindow;

pub fn build(ui: &AppWindow) -> Rc<MonitorController> {
    MonitorController::new(Arc::new(SerialPortLink), ui)
}
