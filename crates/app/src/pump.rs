//! link thread → UI. controller is Rc (not Send): poll on a UI-thread timer

use std::rc::Weak;
use std::time::Duration;

use slint::{Timer, TimerMode};

use crate::modules::monitor::monitor_controller::MonitorController;

// ponytail: 30 ms poll = one UI update per tick at high data rates; event wakeup if latency matters
const TICK: Duration = Duration::from_millis(30);

/// keep the Timer alive; dropping it stops the pump
pub fn start(controller: Weak<MonitorController>) -> Timer {
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, TICK, move || {
        if let Some(controller) = controller.upgrade() {
            controller.pump();
        }
    });
    timer
}
