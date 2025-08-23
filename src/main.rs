use egui::Vec2;
use fx_book_viewer::FxViewerApp;
use log::error;
use log4rs;
use std::process::exit;

fn main() {
    // start log4rs logging framework  - change to tracing?
    if let Err(e) = log4rs::init_file("logging_config.yaml", Default::default()) {
        eprintln!("error initialising log4rs - {e}");
        exit(1);
    }

    // let win_option = eframe::NativeOptions::default();
    let mut fx_viewer_app = FxViewerApp::default();
    let win_option = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(510., 450.)),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "USD/EUR Aggregated Book",
        win_option,
        Box::new(|cc| Ok(Box::new(fx_viewer_app.init(cc)))),
    ) {
        error!("error starting eframe - {e}");
        exit(1);
    }
}
