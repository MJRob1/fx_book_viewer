use egui::Vec2;
use fx_book_viewer::FxViewerApp;
use std::process::exit;

fn main() {
    // let win_option = eframe::NativeOptions::default();
    let win_option = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(505., 450.)),
        // viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(515., 450.)),
        //  viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(1015., 450.)),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "USD/EUR Aggregated Book",
        win_option,
        Box::new(|cc| Ok(Box::new(FxViewerApp::new(cc)))),
    ) {
        eprintln!("error starting eframe - {e}");
        exit(1);
    }
}
