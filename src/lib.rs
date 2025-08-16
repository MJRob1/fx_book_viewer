use eframe::egui;
use egui::{Color32, Context, Label, Layout, RichText};
use egui_extras::{TableBody, TableBuilder, TableRow};
use std::process::exit;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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
    pub fn init(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let ctx = cc.egui_ctx.clone();
        let (ctx_tx, ctx_rx) = mpsc::channel();
        thread::spawn(move || {
            let rec_ctx: Context = ctx_rx.recv().unwrap();
            println!("Got context - now in control thread");

            let (fx_tx, fx_rx) = mpsc::channel();

            thread::spawn(move || {
                println!("now in fx thread");
                loop {
                    let val = String::from("hi - need to change to fx values");
                    thread::sleep(Duration::from_secs(1));
                    if let Err(e) = fx_tx.send(val) {
                        eprintln!("error sending from fx channel - {e}");
                        exit(1);
                    }
                }
            });

            // let received = fx_rx.recv().unwrap();
            for received in fx_rx {
                println!("Got: {received}");
                println!("Repainting display");
                rec_ctx.request_repaint();
                println!("Done display repaint");
            }
        });

        if let Err(e) = ctx_tx.send(ctx) {
            eprintln!("error sending from ctx channel - {e}");
            exit(1);
        }
        // ctx_tx.send(ctx).unwrap();
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
                            render_buy_table_body(body, &fx_book.buy_book);
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
                            render_sell_table_body(body, &fx_book.sell_book);
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

fn render_buy_table_body(mut body: TableBody<'_>, buy_book: &Vec<FxAggBookEntry>) {
    for entry in buy_book {
        let lp_vol_vec = &entry.lp_vol;
        let len = lp_vol_vec.len() - 1;
        let mut lp_vol = String::from("(");
        body.row(30.0, |mut row| {
            row.col(|ui| {
                // ui.label(format!("{:?}", entry.lp_vol));

                let mut index = 0;
                for val in lp_vol_vec {
                    if index == 0 && len == 0 {
                        lp_vol = format!("{}{}: {})", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    } else if index == 0 {
                        lp_vol = format!("{}{}: {},", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    } else if index == len {
                        lp_vol = format!("{} {}: {})", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    } else {
                        lp_vol = format!("{} {}: {},", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    }
                    index += 1;
                }
                ui.label(lp_vol);
            });

            row.col(|ui| {
                ui.label(format!("{:?}", entry.volume));
            });
            row.col(|ui| {
                ui.label(RichText::new(format!("{:?}", entry.price)).color(Color32::GREEN));
            });
        });
    }
}

fn render_sell_table_body(mut body: TableBody<'_>, sell_book: &Vec<FxAggBookEntry>) {
    for entry in sell_book {
        let lp_vol_vec = &entry.lp_vol;
        let len = lp_vol_vec.len() - 1;
        let mut lp_vol = String::from("(");
        body.row(30.0, |mut row| {
            row.col(|ui| {
                ui.label(RichText::new(format!("{:?}", entry.price)).color(Color32::GREEN));
            });

            row.col(|ui| {
                ui.label(format!("{:?}", entry.volume));
            });

            row.col(|ui| {
                // ui.label(format!("{:?}", entry.lp_vol));

                let mut index = 0;
                for val in lp_vol_vec {
                    if index == 0 && len == 0 {
                        lp_vol = format!("{}{}: {})", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    } else if index == 0 {
                        lp_vol = format!("{}{}: {},", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    } else if index == len {
                        lp_vol = format!("{} {}: {})", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    } else {
                        lp_vol = format!("{} {}: {},", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    }
                    index += 1;
                }
                ui.label(lp_vol);
            });
        });
    }
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
                price: 1.5574,
                side: String::from("Buy"),
            },
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("MS "), 3),
                    (String::from("JPMC "), 1),
                    (String::from("CITI "), 5),
                ],
                volume: 9,
                price: 1.5573,
                side: String::from("Buy"),
            },
        ],
        sell_book: vec![FxAggBookEntry {
            lp_vol: vec![(String::from("MS "), 3), (String::from("JPMC "), 5)],
            volume: 8,
            price: 1.5581,
            side: String::from("Sell"),
        }],
        timestamp: 1753430617683973406,
    };

    fx_book
}
