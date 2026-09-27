#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::rc::Rc;

use slint::ComponentHandle;

fn main() -> Result<(), slint::PlatformError> {
    let ui = app::ui::AppWindow::new()?;
    let monitor = app::di::build(&ui);
    // drops before monitor: no tick after the link is joined
    let _pump = app::pump::start(Rc::downgrade(&monitor));
    ui.run()
}
