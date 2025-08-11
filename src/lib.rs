use eframe::egui;
use egui::{Color32, Label, Layout, RichText};
use egui_extras::{TableBody, TableBuilder, TableRow};

#[derive(Debug)]
pub struct FxAggBookEntry {
    pub lp_vol: Vec<(String, i32)>,
    pub volume: i32,
    pub price: f64,
    pub side: String,
}

#[derive(Debug)]
pub struct FxBook {
    pub currency_pair: String,
    pub buy_book: Vec<FxAggBookEntry>,
    pub sell_book: Vec<FxAggBookEntry>,
    pub timestamp: u64,
}

#[derive(Default)]
pub struct FxViewerApp {}

impl FxViewerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for FxViewerApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let fx_book = fx_book_values();
        render_top_panel(self, ctx, frame);
        render_fx_book(self, ctx, frame, &fx_book);
    }
}

fn render_top_panel(
    fx_viewer_app: &mut FxViewerApp,
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
) {
    egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
        ui.with_layout(Layout::left_to_right(eframe::emath::Align::Center), |ui| {
            ui.add_space(134.);
            ui.add(Label::new(
                RichText::new("Buy").text_style(egui::TextStyle::Heading),
            ));
            ui.add_space(162.);
            ui.add(Label::new(
                RichText::new("Sell").text_style(egui::TextStyle::Heading),
            ));
        });
    });
}

fn render_fx_book(
    fx_viewer_app: &mut FxViewerApp,
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    fx_book: &FxBook,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        //   ui.heading("USD/EUR");
        //   ui.label("1.5552");
        ui.with_layout(Layout::left_to_right(eframe::emath::Align::Center), |ui| {
            ui.with_layout(Layout::top_down(eframe::emath::Align::Center), |ui| {
                ui.push_id(1, |ui| {
                    TableBuilder::new(ui)
                        .id_salt(1)
                        .striped(true)
                        .columns(egui_extras::Column::auto().resizable(true), 3)
                        .cell_layout(egui::Layout::default().with_cross_align(egui::Align::Center))
                        .header(20.0, |mut header| {
                            render_buy_table_header(header);
                        })
                        .body(|body| {
                            render_buy_table_body(body);
                        });
                });
            });
            // ui.add_space(10.);
            ui.with_layout(Layout::top_down(eframe::emath::Align::Center), |ui| {
                ui.push_id(2, |ui| {
                    TableBuilder::new(ui)
                        .id_salt(2)
                        .striped(true)
                        .columns(egui_extras::Column::auto().resizable(true), 3)
                        .cell_layout(egui::Layout::default().with_cross_align(egui::Align::Center))
                        .header(20.0, |mut header| {
                            render_sell_table_header(header);
                        })
                        .body(|mut body| {
                            render_sell_table_body(body);
                        });
                });
            });
        });
    });
}

fn render_sell_table_header(mut header: TableRow<'_, '_>) {
    header.col(|ui| {
        ui.heading("Price");
    });
    header.col(|ui| {
        ui.heading("Volume (M)");
    });
    header.col(|ui| {
        ui.heading("");
    });
}

fn render_buy_table_header(mut header: TableRow<'_, '_>) {
    header.col(|ui| {
        ui.heading("");
    });
    header.col(|ui| {
        ui.heading("Volume (M)");
    });
    header.col(|ui| {
        ui.heading("Price");
    });
}

fn render_buy_table_body(mut body: TableBody<'_>) {
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("(CITI: 3, BARX: 3)");
        });
        row.col(|ui| {
            ui.label("6");
        });
        row.col(|ui| {
            ui.label(RichText::new("1.5572").color(Color32::GREEN));
        });
    });
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("(CITI: 1, BARX: 1, BARX: 5)");
        });
        row.col(|ui| {
            ui.label("7");
        });
        row.col(|ui| {
            ui.label(RichText::new("1.5571").color(Color32::GREEN));
        });
    });
}

fn render_sell_table_body(mut body: TableBody<'_>) {
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label(RichText::new("1.5583").color(Color32::GREEN));
        });
        row.col(|ui| {
            ui.label("5");
        });
        row.col(|ui| {
            ui.label("(MS : 5)");
        });
    });
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label(RichText::new("1.5584").color(Color32::GREEN));
        });
        row.col(|ui| {
            ui.label("8");
        });
        row.col(|ui| {
            ui.label("(CITI: 1, BARX: 2, BARX: 5)");
        });
    });
}

fn fx_book_values() -> FxBook {
    let mut fx_book = FxBook {
        currency_pair: String::from(" USD/EUR"),
        buy_book: vec![
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("MS "), 1),
                    (String::from("UBS "), 5),
                    (String::from("CITI "), 3),
                    (String::from("BARX "), 3),
                ],
                volume: 12,
                price: 1.5559,
                side: String::from("Buy"),
            },
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("MS "), 3),
                    (String::from("JPMC "), 1),
                    (String::from("CITI "), 5),
                ],
                volume: 9,
                price: 1.5556,
                side: String::from("Buy"),
            },
        ],
        sell_book: vec![FxAggBookEntry {
            lp_vol: vec![(String::from("MS "), 3), (String::from("JPMC "), 5)],
            volume: 8,
            price: 1.5558,
            side: String::from("Sell"),
        }],
        timestamp: 1753430617683973406,
    };

    fx_book
}
