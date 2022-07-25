#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release mode
#[macro_use]
extern crate lazy_static;
pub mod autosplitters;
pub mod routes;
pub mod usb2snes;

use autosplitters::supermetroid::SNESState;
use eframe::egui;
use egui::containers::ScrollArea;
use livesplit_core::{SharedTimer, Timer};
use parking_lot::RwLock;
use std::error::Error;
use std::sync::Arc;
use std::thread;

#[allow(non_snake_case)]
struct MyApp {
    timer: SharedTimer,
    remaining_space: egui::Vec2,
    timer_precision: Precision,
    // Stored as (avg, stddev), doing it this way lets
    // use a single lock for both
    latency: Arc<RwLock<(f32, f32)>>,
}

impl MyApp {}

fn mk_text(text: &str, size: f32, color: egui::Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.halign = egui::Align::LEFT;
    job.justify = false;
    job.wrap.max_width = f32::INFINITY;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.append(
        text,
        0.0,
        epaint::text::TextFormat {
            font_id: epaint::text::FontId {
                size: size,
                family: epaint::text::FontFamily::Proportional,
            },
            color: color,
            ..epaint::text::TextFormat::default()
        },
    );
    job
}

#[inline]
pub fn columns<R>(
    ui: &mut egui::Ui,
    cols: &[(f32, egui::Layout)],
    add_contents: impl FnOnce(&mut [egui::Ui]) -> R,
) -> R {
    columns_dyn(ui, cols, Box::new(add_contents))
}

fn columns_dyn<'c, R>(
    ui: &mut egui::Ui,
    cols: &[(f32, egui::Layout)],
    add_contents: Box<dyn FnOnce(&mut [egui::Ui]) -> R + 'c>,
) -> R {
    // TODO: ensure there is space
    let spacing = ui.spacing().item_spacing.x;
    let top_left = ui.cursor().min;

    let mut total_width = 0.0;
    let mut columns: Vec<egui::Ui> = vec![];
    for (column_width, col_layout) in cols.iter() {
        let pos = top_left + egui::vec2(total_width, 0.0);
        let child_rect = egui::Rect::from_min_max(
            pos,
            egui::pos2(pos.x + column_width, ui.max_rect().right_bottom().y),
        );
        let mut column_ui = ui.child_ui(child_rect, *col_layout);
        column_ui.set_width(*column_width);
        //total_width += column_ui.min_rect().width();
        total_width += column_width + spacing;
        columns.push(column_ui);
    }

    let result = add_contents(&mut columns[..]);

    let mut max_column_width = cols[0].0;
    let mut max_height = 0.0;
    for column in &columns {
        max_column_width = max_column_width.max(column.min_rect().width());
        max_height = column.min_size().y.max(max_height);
    }

    // Make sure we fit everything next frame:
    //let total_required_width = total_spacing + max_column_width * (num_columns as f32);
    let total_required_width = total_width;

    let size = egui::vec2(ui.available_width().max(total_required_width), max_height);
    ui.allocate_rect(
        egui::Rect::from_min_size(top_left, size),
        egui::Sense {
            click: false,
            drag: false,
            focusable: false,
        },
    );
    result
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark()); // Switch to dark mode
        egui::CentralPanel::default().show(ctx, |ui| {
            //egui::containers::Area::new("area").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(mk_text(
                    &format!(
                        "{} - {}",
                        self.timer.read().run().game_name(),
                        self.timer.read().run().category_name()
                    ),
                    42.0,
                    egui::Color32::WHITE,
                ));
                let latency = self.latency.read();
                ui.label(mk_text(
                    &format!("latency {}ms ± {}ms", latency.0.round(), latency.1.round()),
                    10.0,
                    egui::Color32::GRAY,
                ));
            });
            let time = (self.timer.read().run().offset()
                + self.timer.read().current_attempt_duration())
            .to_duration();
            let time_str = format_time(&time, self.timer_precision);
            let current_split_index = self.timer.read().current_split_index();
            let font_id = epaint::text::FontId {
                size: 32.0,
                family: epaint::text::FontFamily::Proportional,
            };
            let row_height = ui.fonts().row_height(&font_id);
            let timer = self.timer.read();
            let segments = timer.run().segments();
            //let split_index = std::cmp::min(current_split_index.unwrap_or(0), segments.len() - 1);
            //ui.label(segments[split_index].name());
            let total_width = ui.available_width() - ui.spacing().item_spacing.x * 3.0;
            let other_col_width = 180.0;
            let split_col_width = total_width - 2.0 * other_col_width;
            //let split_col_width = 200.0;
            //let other_col_width = (1.0 - split_col_width) / 2.0;
            //let other_col_width = 0.165;
            columns(
                ui,
                &[
                    (
                        split_col_width,
                        egui::Layout::left_to_right().with_cross_align(egui::Align::Min),
                    ),
                    (
                        other_col_width,
                        egui::Layout::right_to_left().with_cross_align(egui::Align::Min),
                    ),
                    (
                        other_col_width,
                        egui::Layout::right_to_left().with_cross_align(egui::Align::Min),
                    ),
                ],
                |col| {
                    col[0].add(egui::Label::new(mk_text("", 32.0, egui::Color32::WHITE)));
                    col[1].add(egui::Label::new(mk_text("PB", 32.0, egui::Color32::WHITE)));
                    col[2].add(egui::Label::new(mk_text(
                        "Time",
                        32.0,
                        egui::Color32::WHITE,
                    )));
                },
            );
            ui.separator();
            ScrollArea::vertical()
                .min_scrolled_height(row_height)
                .max_height((row_height + ui.spacing().item_spacing.y) * 5.0)
                .show_viewport(ui, |ui, _viewport| {
                    for row_index in 0..segments.len() {
                        let this_row_is_highlighted = match current_split_index {
                            None => false,
                            Some(i) => i == row_index,
                        };
                        let row_time = match segments[row_index].split_time().real_time {
                            None => time,
                            Some(rt) => rt.to_duration(),
                        };
                        let row_pb_time = segments[row_index].personal_best_split_time();
                        let time_str = format_time(&row_time, self.timer_precision);
                        let frame = egui::Frame::none();
                        let frame = if this_row_is_highlighted {
                            frame.fill(egui::Color32::BLUE)
                        } else {
                            frame
                        };
                        frame.show(ui, |ui| {
                            columns(
                                ui,
                                &[
                                    (
                                        split_col_width,
                                        egui::Layout::left_to_right()
                                            .with_cross_align(egui::Align::Min),
                                    ),
                                    (
                                        other_col_width,
                                        egui::Layout::right_to_left()
                                            .with_cross_align(egui::Align::Min),
                                    ),
                                    (
                                        other_col_width,
                                        egui::Layout::right_to_left()
                                            .with_cross_align(egui::Align::Min),
                                    ),
                                ],
                                |col| {
                                    // Split name
                                    col[0].label(mk_text(
                                        segments[row_index].name(),
                                        32.0,
                                        egui::Color32::WHITE,
                                    ));
                                    // PB comparison
                                    col[1].scope(|ui| {
                                        match current_split_index {
                                            Some(i)
                                                if row_index < i
                                                    && segments[row_index]
                                                        .split_time()
                                                        .real_time
                                                        .is_some() =>
                                            {
                                                // show comparison
                                                match row_pb_time.real_time {
                                                    None => {
                                                        ui.label(mk_text(
                                                            "",
                                                            32.0,
                                                            egui::Color32::WHITE,
                                                        ));
                                                    }
                                                    Some(rt) => {
                                                        let diff = row_time - rt.to_duration();
                                                        ui.label(mk_text(
                                                            &format!("{}", diff),
                                                            32.0,
                                                            egui::Color32::WHITE,
                                                        ));
                                                    }
                                                };
                                            }
                                            _ => {
                                                ui.label(mk_text("", 32.0, egui::Color32::WHITE));
                                            }
                                        }
                                    });
                                    // Time
                                    col[2].scope(|ui| match current_split_index {
                                        Some(i) if i == row_index => {
                                            ui.label(mk_text(
                                                &time_str,
                                                32.0,
                                                egui::Color32::WHITE,
                                            ));
                                        }
                                        Some(i)
                                            if row_index < i
                                                && segments[row_index]
                                                    .split_time()
                                                    .real_time
                                                    .is_some() =>
                                        {
                                            ui.label(mk_text(
                                                &time_str,
                                                32.0,
                                                egui::Color32::WHITE,
                                            ));
                                        }
                                        _ => {
                                            ui.label(mk_text("", 32.0, egui::Color32::WHITE));
                                            //ui.label(mk_text(&time_str, 32.0, egui::Color32::WHITE));
                                        }
                                    });
                                },
                            );
                        });
                        if this_row_is_highlighted {
                            //ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                            ui.scroll_to_cursor(Some(egui::Align::Center));
                            //ui.scroll_to_cursor(None);
                        }
                    }
                });
            ui.separator();
            ui.with_layout(
                egui::Layout::right_to_left().with_cross_align(egui::Align::Min),
                |ui| {
                    ui.label(mk_text(&time_str, 42.0, egui::Color32::GREEN));
                },
            );
            self.remaining_space = ui.available_size_before_wrap();
        });
        let mut sz = ctx.input().screen_rect.size();
        sz.y -= self.remaining_space.y;
        frame.set_window_size(sz);
        //println!("sz = {:#?}", sz);
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Precision {
    Seconds,
    TenthsOfSeconds,
    HundredthsOfSeconds,
}

fn format_time(time: &time::Duration, timer_precision: Precision) -> String {
    let mut time_str = "".to_owned();

    if time.whole_hours() > 0 {
        time_str += &format!("{}:", time.whole_hours());
    }

    if time.whole_hours() > 0 && time.whole_minutes() > 0 {
        time_str += &format!("{:02}:", time.whole_minutes() % 60);
    } else if time.whole_minutes() > 0 {
        time_str += &format!("{}:", time.whole_minutes() % 60);
    }

    if time.whole_minutes() > 0 && time.whole_seconds() > 0 {
        time_str += &format!("{:02}", time.whole_seconds() % 60);
    } else {
        time_str += &format!("{}", time.whole_seconds() % 60);
    }

    if time.subsec_milliseconds() > 0 {
        match timer_precision {
            Precision::Seconds => {}
            Precision::TenthsOfSeconds => {
                time_str += "";
                let ms_str = format!("{:01}", time.subsec_milliseconds());
                time_str += &format!(".{ms:.*}", 1, ms = ms_str);
            }
            Precision::HundredthsOfSeconds => {
                let ms_str = format!("{:02}", time.subsec_milliseconds());
                time_str += &format!(".{ms:.*}", 2, ms = ms_str);
            }
        }
    }

    time_str
}

fn print_on_error<F>(f: F)
where
    F: FnOnce() -> std::result::Result<(), Box<dyn Error>>,
{
    match f() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    let polling_rate = 20.0;
    let frame_rate = 30.0;
    let (settings, run) = routes::supermetroid::hundo();
    //let (settings, run) = routes::supermetroid::anypercent();
    let timer = Timer::new(run)
        .expect("Run with at least one segment provided")
        .into_shared();
    let options = eframe::NativeOptions {
        always_on_top: true,
        // TODO: fix me
        initial_window_size: Some(egui::vec2(470.0, 337.0)),
        ..eframe::NativeOptions::default()
    };
    println!("size = {:#?}", options.initial_window_size);
    let latency = Arc::new(RwLock::new((0.0, 0.0)));

    let app = MyApp {
        timer: timer.clone(),
        remaining_space: egui::Vec2 { x: 0.0, y: 0.0 },
        timer_precision: Precision::TenthsOfSeconds,
        latency: latency.clone(),
    };
    eframe::run_native(
        "Annelid",
        options,
        Box::new(move |cc| {
            let context = cc.egui_ctx.clone();
            // This thread is essentially just a refresh rate timer
            // it ensures that the gui thread is redrawn at the requested frame_rate,
            // possibly more often.
            let _frame_rate_thread = thread::spawn(move || loop {
                context.request_repaint();
                std::thread::sleep(std::time::Duration::from_millis(
                    (1000.0 / frame_rate) as u64,
                ));
            });
            // This thread deals with polling the SNES at a fixed rate.
            let _snes_polling_thread = thread::spawn(move || loop {
                let timer = timer.clone();
                let settings = settings.clone();
                let latency = latency.clone();
                print_on_error(move || -> std::result::Result<(), Box<dyn Error>> {
                    let mut client = usb2snes::SyncClient::connect();
                    client.set_name("annelid".to_owned())?;
                    println!("Server version is {:?}", client.app_version()?);
                    let mut devices = client.list_device()?;
                    if devices.len() != 1 {
                        if devices.len() < 1 {
                            Err("No devices present")?;
                        } else {
                            Err(format!("You need to select a device: {:#?}", devices))?;
                        }
                    }
                    let device = devices.pop().ok_or("Device list was empty")?;
                    println!("Using device: {}", device);
                    client.attach(&device)?;
                    println!("Connected.");
                    println!("{:#?}", client.info()?);
                    let mut snes = SNESState::new();
                    loop {
                        let summary = snes.fetch_all(&mut client, &settings)?;
                        if summary.start {
                            timer.write().start();
                        }
                        if summary.reset {
                            timer.write().reset(true);
                            snes = SNESState::new();
                        }
                        if summary.split {
                            timer.write().split();
                        }
                        {
                            *latency.write() = (summary.latency_average, summary.latency_stddev);
                        }
                        std::thread::sleep(std::time::Duration::from_millis(
                            (1000.0 / polling_rate) as u64,
                        ));
                    }
                });
                std::thread::sleep(std::time::Duration::from_millis(1000));
            });

            Box::new(app)
        }),
    );
}
