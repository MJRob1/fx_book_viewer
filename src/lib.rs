pub mod aggregator;
pub mod simulator;
use crate::aggregator::{FxAggBookEntry, FxBook};
use crate::simulator::Config;
use eframe::egui;
use egui::{Color32, Context, Label, Layout, RichText};
use egui_extras::{TableBody, TableBuilder, TableRow};
use log::error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::prelude::*;
use std::num::ParseFloatError;
use std::num::ParseIntError;
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;

#[derive(Debug)]
#[non_exhaustive]
pub enum AppError {
    NumParams,
    IsEmpty,
    ParseFloat(ParseFloatError),
    ParseInt(ParseIntError),
    Io(io::Error),
}

impl From<ParseFloatError> for AppError {
    fn from(error: ParseFloatError) -> Self {
        Self::ParseFloat(error)
    }
}

impl From<ParseIntError> for AppError {
    fn from(error: ParseIntError) -> Self {
        Self::ParseInt(error)
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IsEmpty => f.write_str("empty data field"),
            Self::NumParams => f.write_str("missing market data fields"),
            Self::ParseFloat(e) => Display::fmt(e, f),
            Self::ParseInt(e) => Display::fmt(e, f),
            Self::Io(e) => Display::fmt(e, f),
        }
    }
}

impl std::error::Error for AppError {}
/*
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
    pub fn new(config: &Vec<Config>) -> Self {
        // create a new FxBook with empty buy and sell books
        // and a timestamp of current time
        // using first config entry to get currency pair
        let currency_pair = config[0].currency_pair.clone();
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
*/
pub fn run<F: Future>(future: F) -> F::Output {
    let rt = Runtime::new().unwrap();
    rt.block_on(future)
}

pub fn create_log_file(file_path: &str) -> Result<BufWriter<File>, AppError> {
    let path = Path::new(file_path);

    // Open a file in write-only mode, returns `io::Result<File>`
    let file = File::create(&path)?;

    Ok(BufWriter::new(file))
}

pub fn write_to_fix_log(
    writer: &mut BufWriter<File>,
    market_data: &String,
) -> Result<(), AppError> {
    writeln!(writer, "{}", market_data)?;
    Ok(())
}

pub fn get_params(data: &str, number: usize) -> Result<std::str::Split<'_, &str>, AppError> {
    let value = data.split("|");
    if value.clone().count() < number {
        return Err(AppError::NumParams);
    } else {
        Ok(data.split("|"))
    }
}

pub fn get_str_field(field: Option<&str>) -> Result<&str, AppError> {
    let value = field.unwrap_or("");
    if value.trim().is_empty() {
        return Err(AppError::IsEmpty);
    } else {
        Ok(value.trim())
    }
}

/*
pub fn fx_book_copy(fx_book: FxBook) -> FxBook {
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
*/

#[derive(Default, Debug)]
pub struct FxViewerApp {
    pub fx_book_mutex: Arc<Mutex<FxBook>>,
    // pub fx_book: FxBook,
}

impl FxViewerApp {
    pub fn init(&mut self, cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = cc.egui_ctx.clone();
        let (ctx_tx, ctx_rx) = mpsc::channel();
        // Create "FIX" log file
        let mut writer = match create_log_file("logs/fix.log") {
            Ok(writer) => writer,
            Err(e) => {
                error!("problem creating log file - {e}");
                exit(1);
            }
        };
        //   let (fx_tx, fx_rx) = mpsc::channel();
        // read config file to get configs for each liquidity provider source
        let mut configs: Vec<Config> = Vec::new();
        if let Err(e) = simulator::get_configs(&mut configs) {
            error!("config input file not processed - {e}");
            exit(1);
        }
        // Create aggregated FX Book
        let mut fx_book = FxBook::new(&configs);

        let fx_book_mutex = Arc::new(Mutex::new(fx_book));
        let fx_book_mutex_ui_clone = Arc::clone(&fx_book_mutex);
        let fx_book_mutex_fx_clone = Arc::clone(&fx_book_mutex);
        thread::spawn(move || {
            // start fx thread
            let rec_ctx: Context = ctx_rx.recv().unwrap();

            run(async {
                /*  async returns a future rather than blocking current thread
                run() starts a runtime and hands the future to the runtime all the code - the entire program
                is the signature future argument of run! Note: everything inside the async code avoids blocking
                but any code outside run will block on the run function returning */

                // Combine all individual market data streams from each liquidity provider into a single merged stream
                // that yields values in the order they arrive from the source market data streams
                let mut merged_streams_map = simulator::start_streams(&configs);

                while let Some(val) = merged_streams_map.next().await {
                    // await polls the future until future returns Ready.
                    // If future still pending then control is handed to the runtime
                    let (_key, market_data) = val;

                    // write market data to a "FIX" log
                    if let Err(e) = write_to_fix_log(&mut writer, &market_data) {
                        error!("problem writing to FIX log - {e}");
                    }

                    // Update the Fx Book with the new market data
                    let mut fx_book = fx_book_mutex_fx_clone.lock().unwrap();
                    if let Err(e) = fx_book.update(market_data) {
                        error!("market data not processed - {e}");
                    } else {
                        // currently working on real-time GUI rather than print!!
                        aggregator::print_fxbook_as_ladder(&mut fx_book);
                    }
                    rec_ctx.request_repaint();
                }
            });

            /*    let mut fx_book = fx_book_mutex_fx_clone.lock().unwrap();
                        *fx_book = fx_book_values(); // fx book will be main fx book
                        println!("fx_thread1: fx_book: {:?}", fx_book);
                        rec_ctx.request_repaint();
                        // test - send second fxbook update
                        *fx_book = fx_book_values2(); // fx book will be main fx book
                        println!("fx_thread2: fx_book: {:?}", fx_book);
                        rec_ctx.request_repaint();
            */
            // }
        }); // end of fx thread  - mutex lock released here

        if let Err(e) = ctx_tx.send(ctx) {
            error!("error sending from ctx channel - {e}");
            exit(1);
        }

        Self {
            fx_book_mutex: fx_book_mutex_ui_clone,
        }
    } // mutex lock released here
}

impl eframe::App for FxViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        render_top_panel(ctx);
        render_fx_book(self, ctx);
    }
}

fn render_top_panel(ctx: &egui::Context) {
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

fn render_fx_book(fx_viewer_app: &mut FxViewerApp, ctx: &egui::Context) {
    let fx_book = fx_viewer_app.fx_book_mutex.lock().unwrap(); // panic if can't get lock
    //  println!("render_fx_book: fx_book: {:?}", fx_book);
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(Layout::left_to_right(eframe::emath::Align::Center), |ui| {
            ui.with_layout(Layout::top_down(eframe::emath::Align::Center), |ui| {
                ui.push_id(1, |ui| {
                    TableBuilder::new(ui)
                        .id_salt(1)
                        .striped(true)
                        .columns(egui_extras::Column::auto().resizable(true), 3)
                        .cell_layout(egui::Layout::default().with_cross_align(egui::Align::Center))
                        .header(20.0, |header| {
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
                        .header(20.0, |header| {
                            render_sell_table_header(header);
                        })
                        .body(|body| {
                            render_sell_table_body(body, &fx_book.sell_book);
                        });
                });
            });
        });
    });
} // mutex lock released here

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

fn fx_book_values2() -> FxBook {
    let fx_book = FxBook {
        currency_pair: String::from(" USD/EUR"),
        buy_book: vec![
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("MS "), 1),
                    (String::from("UBS "), 5),
                    (String::from("CITI "), 3),
                ],
                volume: 8,
                price: 1.5555,
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
            FxAggBookEntry {
                lp_vol: vec![(String::from("UBS "), 1)],
                volume: 1,
                price: 1.5553,
                side: String::from("Buy"),
            },
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("UBS "), 3),
                    (String::from("CITI "), 1),
                    (String::from("BARX "), 1),
                    (String::from("BARX "), 5),
                ],
                volume: 10,
                price: 1.5554,
                side: String::from("Buy"),
            },
        ],
        sell_book: vec![
            FxAggBookEntry {
                lp_vol: vec![(String::from("MS "), 3), (String::from("JPMC "), 5)],
                volume: 8,
                price: 1.5565,
                side: String::from("Sell"),
            },
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("UBS "), 3),
                    (String::from("CITI "), 3),
                    (String::from("BARX "), 3),
                ],
                volume: 9,
                price: 1.5563,
                side: String::from("Sell"),
            },
            FxAggBookEntry {
                lp_vol: vec![(String::from("JPMC "), 1)],
                volume: 1,
                price: 1.5567,
                side: String::from("Sell"),
            },
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("MS "), 5),
                    (String::from("UBS "), 1),
                    (String::from("CITI "), 1),
                    (String::from("BARX "), 1),
                    (String::from("BARX "), 5),
                ],
                volume: 13,
                price: 1.5564,
                side: String::from("Sell"),
            },
            FxAggBookEntry {
                lp_vol: vec![(String::from("MS "), 1), (String::from("JPMC "), 3)],
                volume: 4,
                price: 1.5566,
                side: String::from("Sell"),
            },
        ],
        timestamp: 1753430617683973406,
    };

    fx_book
}
