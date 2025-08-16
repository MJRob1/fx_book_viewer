use egui::Vec2;
use fx_book_viewer::FxViewerApp;
use std::process::exit;
use std::sync::mpsc;
use std::thread;

fn main() {
    // let win_option = eframe::NativeOptions::default();
    let win_option = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(510., 450.)),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "USD/EUR Aggregated Book",
        win_option,
        Box::new(|cc| Ok(Box::new(FxViewerApp::init(cc)))),
    ) {
        eprintln!("error starting eframe - {e}");
        exit(1);
    }
}
