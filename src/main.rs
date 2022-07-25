#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release mode
#[macro_use]
extern crate lazy_static;
pub mod autosplitters;
pub mod routes;
pub mod usb2snes;

use autosplitters::supermetroid::{SNESState, Settings};
use eframe::egui;
use egui::containers::ScrollArea;
use livesplit_core::Timer;
use std::collections::VecDeque;
use std::error::Error;
use std::time::Instant;

#[allow(non_snake_case)]
struct MyApp {
    client: usb2snes::SyncClient,
    snes: SNESState,
    settings: Settings,
    timer: Timer,
    latency_samples: VecDeque<u128>,
    remaining_space: egui::Vec2,
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
        ctx.set_visuals(egui::Visuals::dark()); // Switch to light mode
        let start = Instant::now();
        match self.snes.fetch_all(&mut self.client) {
            Err(e) => {
                println!("{}", e);
                frame.quit();
                return;
            }
            Ok(()) => {}
        }
        let elapsed = start.elapsed().as_millis();
        self.latency_samples.push_back(elapsed);
        if self.latency_samples.len() > 1000 {
            self.latency_samples.pop_front();
        }
        let average_latency: u128 =
            self.latency_samples.iter().sum::<u128>() / self.latency_samples.len() as u128;
        let mut s = 0;
        for x in self.latency_samples.iter() {
            let y = *x as i128;
            let avg = average_latency as i128;
            let diff = y - avg;
            s += diff * diff;
        }
        let stddev = (s as f64 / (self.latency_samples.len() as f64 - 1f64)).sqrt();
        if self.snes.start() {
            self.timer.start();
        }
        if self.snes.reset() {
            self.timer.reset(true);
        }
        if autosplitters::supermetroid::split(&self.settings, &mut self.snes) {
            self.timer.split();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            //egui::containers::Area::new("area").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(mk_text(
                    &format!(
                        "{} - {}",
                        self.timer.run().game_name(),
                        self.timer.run().category_name()
                    ),
                    42.0,
                    egui::Color32::WHITE,
                ));
                ui.label(mk_text(
                    &format!("latency {}ms ± {:02}ms", average_latency, stddev.round()),
                    10.0,
                    egui::Color32::GRAY,
                ));
            });
            let time =
                (self.timer.run().offset() + self.timer.current_attempt_duration()).to_duration();
            let ms_str = format!("{:02}", time.subsec_milliseconds());
            let time_str = format!(
                "{:02}:{:02}:{:02}.{ms:.*}",
                time.whole_hours(),
                time.whole_minutes() % 60,
                time.whole_seconds() % 60,
                2,
                ms = ms_str
            );
            let current_split_index = self.timer.current_split_index();
            let font_id = epaint::text::FontId {
                size: 32.0,
                family: epaint::text::FontFamily::Proportional,
            };
            let row_height = ui.fonts().row_height(&font_id);
            let segments = self.timer.run().segments();
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
                        //let row_time = livesplit_core::TimeSpan::zero().to_duration();
                        let row_pb_time = segments[row_index].personal_best_split_time();
                        let ms_str = format!("{:02}", row_time.subsec_milliseconds());
                        let time_str = format!(
                            "{:02}:{:02}:{:02}.{ms:.*}",
                            row_time.whole_hours(),
                            row_time.whole_minutes() % 60,
                            row_time.whole_seconds() % 60,
                            2,
                            ms = ms_str
                        );
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
        ctx.request_repaint();
        let mut sz = ctx.input().screen_rect.size();
        sz.y -= self.remaining_space.y;
        frame.set_window_size(sz);
        //println!("sz = {:#?}", sz);
    }
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    let mut client = usb2snes::SyncClient::connect();
    client.set_name("annelid".to_owned())?;
    println!("Server version is {:?}", client.app_version());
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
    let options = eframe::NativeOptions {
        always_on_top: true,
        // TODO: fix me
        initial_window_size: Some(egui::vec2(470.0, 337.0)),
        ..eframe::NativeOptions::default()
    };
    println!("size = {:#?}", options.initial_window_size);
    let mut snes = SNESState::new();
    // We need to initialize the memory state before entering the polling loop
    snes.fetch_all(&mut client)?;

    let (settings, run) = routes::supermetroid::hundo();
    //let (settings, run) = routes::supermetroid::anypercent();

    let app = MyApp {
        client: client,
        snes: snes,
        settings: settings,
        latency_samples: VecDeque::from([]),
        timer: Timer::new(run).expect("Run with at least one segment provided"),
        remaining_space: egui::Vec2 { x: 0.0, y: 0.0 },
    };
    eframe::run_native("Annelid", options, Box::new(|_cc| Box::new(app)));
}
