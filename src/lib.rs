use eframe::egui;
use egui::{Color32, Context, Label, Layout, RichText};
use egui_extras::{TableBody, TableBuilder, TableRow};
use std::process::exit;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct FxAggBookEntry {
    pub lp_vol: Vec<(String, i32)>,
    pub volume: i32,
    pub price: f64,
    pub side: String,
}
impl Clone for FxAggBookEntry {
    fn clone(&self) -> Self {
        let mut lp_vol: Vec<(String, i32)> = Vec::new();

        for val in self.lp_vol.iter() {
            lp_vol.push(val.clone());
        }
        Self {
            lp_vol,
            volume: self.volume.clone(),
            price: self.price.clone(),
            side: self.side.clone(),
        }
    }
}

#[derive(Debug, Default)]
pub struct FxBook {
    pub currency_pair: String,
    pub buy_book: Vec<FxAggBookEntry>,
    pub sell_book: Vec<FxAggBookEntry>,
    pub timestamp: u64,
}

impl FxBook {
    pub fn new() -> Self {
        let currency_pair = String::from("USD/EUR");
        let buy_book: Vec<FxAggBookEntry> = Vec::new();
        let sell_book: Vec<FxAggBookEntry> = Vec::new();
        //need to catch this possible panic on unwrap when converting u126 to u64
        let timestamp: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .try_into()
            .unwrap();

        FxBook {
            currency_pair,
            buy_book,
            sell_book,
            timestamp,
        }
    }
    pub fn copy(&mut self) -> Self {
        // create a copy of the FxBook
        let mut buy_book: Vec<FxAggBookEntry> = Vec::new();
        for val in self.buy_book.iter() {
            buy_book.push(val.clone());
        }

        let mut sell_book: Vec<FxAggBookEntry> = Vec::new();
        for val in self.sell_book.iter() {
            sell_book.push(val.clone());
        }
        FxBook {
            currency_pair: self.currency_pair.clone(),
            buy_book,
            sell_book,
            timestamp: self.timestamp,
        }
    }
}

pub fn fx_book_copy(fx_book: &FxBook) -> FxBook {
    // create a copy of the FxBook
    let mut buy_book: Vec<FxAggBookEntry> = Vec::new();
    for val in fx_book.buy_book.iter() {
        buy_book.push(val.clone());
    }

    let mut sell_book: Vec<FxAggBookEntry> = Vec::new();
    for val in fx_book.sell_book.iter() {
        sell_book.push(val.clone());
    }
    FxBook {
        currency_pair: fx_book.currency_pair.clone(),
        buy_book,
        sell_book,
        timestamp: fx_book.timestamp,
    }
}
#[derive(Default, Debug)]
pub struct FxViewerApp {
    // pub fx_book_ref: Arc<Mutex<FxBook>>,
    pub fx_book: FxBook,
}

impl FxViewerApp {
    pub fn init(&mut self, cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let ctx = cc.egui_ctx.clone();
        let (ctx_tx, ctx_rx) = mpsc::channel();
        let (fx_tx, fx_rx) = mpsc::channel();
        thread::spawn(move || {
            // start fx thread
            let rec_ctx: Context = ctx_rx.recv().unwrap();

            //  loop {
            let mut fx_book = fx_book_values(); // fx book will be main fx book
            //   println!("fx_thread: fx_book: {:?}", fx_book);
            let fx_book_copy = fx_book.copy(); // this is copy which is sent to ui
            //  println!("fx_thread: fx_book_copy: {:?}", fx_book_copy);

            if let Err(e) = fx_tx.send(fx_book_copy) {
                eprintln!("error sending from fx channel - {e}");
                exit(1);
            }

            //  println!("Repainting display");
            rec_ctx.request_repaint();

            // }
        }); // end of fx thread  

        if let Err(e) = ctx_tx.send(ctx) {
            eprintln!("error sending from ctx channel - {e}");
            exit(1);
        }

        let mut fx_book_update = fx_rx.recv().unwrap(); // catch panic?
        // for received in fx_rx {
        // }

        Self {
            fx_book: fx_book_update,
        }
    }
}

impl eframe::App for FxViewerApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let fx_book = fx_book_copy(&self.fx_book);
        //  println!("update: fx_book: {:?}", fx_book);
        render_top_panel(self, ctx, frame);
        render_fx_book(self, ctx, frame, fx_book);
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
    fx_book: FxBook,
) {
    // let fx_book = *fx_viewer_app.fx_book_ref.lock().unwrap(); // panic if lock error
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
    let fx_book = FxBook {
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
