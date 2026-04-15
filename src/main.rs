mod document;
mod i18n;
mod paths;
mod ui;

use adw::prelude::*;
use gtk::gio;

const APP_ID: &str = "org.zugferd.QubaViewer";

fn main() {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_startup(ui::build_startup);
    app.connect_activate(ui::build_ui);
    app.connect_open(ui::open_files);
    app.run();
}
