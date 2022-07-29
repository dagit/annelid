#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release mode
#[macro_use]
extern crate lazy_static;
pub mod autosplitters;
pub mod routes;
pub mod usb2snes;

use autosplitters::supermetroid::{SNESState, Settings};
use eframe::egui;
use livesplit_core::layout::{ComponentSettings, LayoutSettings};
use livesplit_core::{Layout, Run, Segment, SharedTimer, Timer};
use parking_lot::RwLock;
use std::error::Error;
use std::sync::Arc;
use std::thread;

fn messagebox_on_error<F>(f: F)
where
    F: FnOnce() -> std::result::Result<(), Box<dyn Error>>,
{
    use native_dialog::{MessageDialog, MessageType};
    match f() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", e);
            MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title("Error")
                .set_text(&format!("{}", e))
                .show_alert()
                .unwrap();
        }
    }
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

struct LiveSplitCoreRenderer {
    texture: Option<egui::TextureHandle>,
    layout: Layout,
    timer: SharedTimer,
    renderer: livesplit_core::rendering::software::Renderer,
    show_settings_editor: bool,
    settings: Arc<RwLock<Settings>>,
}

fn show_children(
    settings: &mut Settings,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    roots: &mut [String],
) {
    roots.sort();
    roots.iter().for_each(|key| {
        let mut children = settings.children(key);
        let id = ui.make_persistent_id(key);
        if !children.is_empty() {
            egui::collapsing_header::CollapsingState::load_with_default_open(ctx, id, false)
                .show_header(ui, |ui| {
                    ui.checkbox(settings.lookup_mut(key), key);
                })
                .body(|ui| {
                    ui.indent(id, |ui| {
                        ui.scope(|ui| {
                            ui.set_enabled(settings.lookup(key));
                            show_children(settings, ui, ctx, &mut children);
                        });
                    });
                });
        } else {
            ui.scope(|ui| {
                ui.set_enabled(true);
                ui.checkbox(settings.lookup_mut(key), key);
            });
        }
    });
}

impl eframe::App for LiveSplitCoreRenderer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark()); // Switch to dark mode
        let settings_editor = egui::containers::Window::new("Settings Editor");
        egui::Area::new("livesplit")
            .enabled(!self.show_settings_editor)
            .show(ctx, |ui| {
                let sz = ctx.input().screen_rect.size();
                let texture: &mut egui::TextureHandle = self.texture.get_or_insert_with(|| {
                    let sz = [sz.x as usize, sz.y as usize];
                    let buffer = vec![0; 4 * sz[0] * sz[1]];
                    let blank = egui::ColorImage::from_rgba_unmultiplied(sz, buffer.as_slice());
                    ui.ctx().load_texture("frame", blank)
                });

                // a local scope so the timer lock has a smaller scope
                let layout_state = {
                    let timer = self.timer.read();
                    let snapshot = timer.snapshot();
                    self.layout.state(&snapshot)
                };
                let sz_vec2 = [sz.x as f32, sz.y as f32];

                let szu32 = [sz.x as u32, sz.y as u32];
                let sz = [sz.x as usize, sz.y as usize];
                self.renderer.render(&layout_state, szu32);
                let raw_frame = self.renderer.image_data();
                // Note: Don't use from_rgba_unmultiplied() here. It's super slow.
                let pixels = raw_frame
                    .chunks_exact(4)
                    .map(|p| egui::Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
                    .collect();
                let raw_frame = epaint::image::ColorImage { size: sz, pixels };

                texture.set(raw_frame);
                ui.image(texture.id(), sz_vec2);
            })
            .response
            .context_menu(|ui| {
                use native_dialog::FileDialog;
                let empty_path = "".to_owned();
                let document_dir = match directories::UserDirs::new() {
                    None => empty_path,
                    Some(d) => match d.document_dir() {
                        None => empty_path,
                        Some(d) => d.to_str().unwrap_or("").to_owned(),
                    },
                };
                ui.menu_button("LiveSplit Save/Load", |ui| {
                    if ui.button("Import Layout").clicked() {
                        ui.close_menu();
                        messagebox_on_error(|| {
                            let path = FileDialog::new()
                                .set_location(&document_dir)
                                .add_filter("LiveSplit Layout", &["lsl"])
                                .add_filter("Any file", &["*"])
                                .show_open_single_file()?;
                            let path = match path {
                                Some(path) => path,
                                None => return Ok(()),
                            };
                            let f = std::fs::File::open(path.clone())?;
                            self.layout =
                                livesplit_core::layout::parser::parse(std::io::BufReader::new(f))?;
                            let layout_file = std::fs::read_to_string(path)?;
                            let doc = roxmltree::Document::parse(&layout_file)?;
                            doc.root().children().for_each(|d| {
                                if d.tag_name().name() == "Layout" {
                                    use std::str::FromStr;
                                    let mut mode = None;
                                    let mut x = None;
                                    let mut y = None;
                                    let mut width = None;
                                    let mut height = None;
                                    d.children().for_each(|d| {
                                        if d.tag_name().name() == "Mode" {
                                            mode = d.text();
                                        }
                                        if d.tag_name().name() == "X" {
                                            x = d.text().and_then(|d| f32::from_str(d).ok());
                                        }
                                        if d.tag_name().name() == "Y" {
                                            y = d.text().and_then(|d| f32::from_str(d).ok());
                                        }
                                        if mode.is_some() && d.tag_name().name() == format!("{}Width", mode.unwrap()) {
                                            width = d.text().and_then(|d| f32::from_str(d).ok());
                                        }
                                        if mode.is_some() && d.tag_name().name() == format!("{}Height", mode.unwrap()) {
                                            height = d.text().and_then(|d| f32::from_str(d).ok());
                                        }
                                        if let (Some(x), Some(y), Some(width), Some(height)) = (x,y,width,height) {
                                                frame.set_window_size(egui::Vec2::new(width, height));
                                                frame.set_window_pos(egui::Pos2::new(x,y));
                                        }
                                    });
                                }
                            });

                            Ok(())
                        });
                    }
                    if ui.button("Import Splits").clicked() {
                        ui.close_menu();
                        messagebox_on_error(|| {
                            use livesplit_core::run::parser::composite;
                            let path = FileDialog::new()
                                .set_location(&document_dir)
                                .add_filter("LiveSplit Splits", &["lss"])
                                .add_filter("Any file", &["*"])
                                .show_open_single_file()?;
                            let path = match path {
                                Some(path) => path,
                                None => return Ok(()),
                            };
                            let f = std::fs::File::open(path.clone())?;
                            *self.timer.write() = Timer::new(
                                composite::parse(
                                    std::io::BufReader::new(f),
                                    path.parent().map(|p| p.to_path_buf()),
                                    true,
                                )?
                                .run,
                            )?;
                            Ok(())
                        });
                    }
                    if ui.button("Save Splits as...").clicked() {
                        ui.close_menu();
                        // TODO: refactor this to a function
                        messagebox_on_error(|| {
                            let mut fname = self.timer.read().run().extended_file_name(false);
                            if fname.is_empty() {
                                fname += "annelid.lss";
                            } else {
                                fname += ".lss";
                            }
                            let path = FileDialog::new()
                                .set_location(&document_dir)
                                .set_filename(&fname)
                                .add_filter("LiveSplit Splits", &["lss"])
                                .add_filter("Any file", &["*"])
                                .show_save_single_file()?;
                            let path = match path {
                                Some(path) => path,
                                None => return Ok(()),
                            };
                            let f = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .truncate(true)
                                .open(path)?;
                            let writer = std::io::BufWriter::new(f);
                            livesplit_core::run::saver::livesplit::save_timer(
                                &*self.timer.read(),
                                writer,
                            )?;
                            self.timer.write().mark_as_unmodified();
                            Ok(())
                        });
                    }
                });
                ui.menu_button("Run Control", |ui| {
                    if ui.button("Start").clicked() {
                        self.timer.write().start();
                        ui.close_menu()
                    }
                    if ui.button("Split").clicked() {
                        self.timer.write().split();
                        ui.close_menu()
                    }
                    ui.separator();
                    if ui.button("Skip Split").clicked() {
                        self.timer.write().skip_split();
                        ui.close_menu()
                    }
                    if ui.button("Undo Split").clicked() {
                        self.timer.write().undo_split();
                        ui.close_menu()
                    }
                    ui.separator();
                    if ui.button("Pause").clicked() {
                        self.timer.write().pause();
                        ui.close_menu()
                    }

                    if ui.button("Resume").clicked() {
                        self.timer.write().resume();
                        ui.close_menu()
                    }
                    ui.separator();
                    if ui.button("Reset").clicked() {
                        // TODO: this should also tell the snes watcher thread
                        // to create a new snes state
                        self.timer.write().reset(true);
                        ui.close_menu()
                    }
                });
                ui.menu_button("Autosplitter", |ui| {
                    if ui.button("Configure").clicked() {
                        self.show_settings_editor = true;
                        ui.close_menu();
                    }
                    if ui.button("Load Configuration").clicked() {
                        ui.close_menu();
                        messagebox_on_error(|| {
                            let path = FileDialog::new()
                                .set_location(&document_dir)
                                .add_filter("Autosplitter Configuration", &["asc"])
                                .add_filter("Any file", &["*"])
                                .show_open_single_file()?;
                            let path = match path {
                                Some(path) => path,
                                None => return Ok(()),
                            };
                            let f = std::fs::File::open(path)?;
                            *self.settings.write() =
                                serde_json::from_reader(std::io::BufReader::new(f))?;
                            Ok(())
                        });
                    }
                    if ui.button("Save Configuration").clicked() {
                        ui.close_menu();
                        messagebox_on_error(|| {
                            let mut fname = self.timer.read().run().extended_file_name(false);
                            if fname.is_empty() {
                                fname += "annelid.asc";
                            } else {
                                fname += ".asc";
                            }
                            let path = FileDialog::new()
                                .set_location(&document_dir)
                                .set_filename(&fname)
                                .add_filter("Autosplitter Configuration", &["asc"])
                                .add_filter("Any file", &["*"])
                                .show_save_single_file()?;
                            let path = match path {
                                Some(path) => path,
                                None => return Ok(()),
                            };
                            let f = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .truncate(true)
                                .open(path)?;
                            serde_json::to_writer(&f, &*self.settings.read())?;
                            Ok(())
                        });
                    }
                });
                ui.separator();
                if ui.button("Quit").clicked() {
                    use native_dialog::{MessageDialog, MessageType};
                    if self.timer.read().run().has_been_modified() {
                        let save_requested = MessageDialog::new()
                            .set_type(MessageType::Error)
                            .set_title("Error")
                            .set_text("Splits have been modified. Save splits?")
                            .show_confirm()
                            .unwrap();
                        if save_requested {
                            messagebox_on_error(|| {
                                let mut fname = self.timer.read().run().extended_file_name(false);
                                if fname.is_empty() {
                                    fname += "annelid.lss";
                                } else {
                                    fname += ".lss";
                                }
                                let path = FileDialog::new()
                                    .set_location(&document_dir)
                                    .set_filename(&fname)
                                    .add_filter("LiveSplit Splits", &["lss"])
                                    .add_filter("Any file", &["*"])
                                    .show_save_single_file()?;
                                let path = match path {
                                    Some(path) => path,
                                    None => return Ok(()),
                                };
                                let f = std::fs::OpenOptions::new()
                                    .create(true)
                                    .write(true)
                                    .truncate(true)
                                    .open(path)?;
                                let writer = std::io::BufWriter::new(f);
                                livesplit_core::run::saver::livesplit::save_timer(
                                    &*self.timer.read(),
                                    writer,
                                )?;
                                Ok(())
                            });
                        }
                    }
                    if self.settings.read().has_been_modified() {
                        let save_requested = MessageDialog::new()
                            .set_type(MessageType::Error)
                            .set_title("Error")
                            .set_text("Autosplit config may have been modified. Save autosplitter config?")
                            .show_confirm()
                            .unwrap();
                        if save_requested {
                            messagebox_on_error(|| {
                                let mut fname = self.timer.read().run().extended_file_name(false);
                                if fname.is_empty() {
                                    fname += "annelid.asc";
                                } else {
                                    fname += ".asc";
                                }
                                let path = FileDialog::new()
                                    .set_location(&document_dir)
                                    .set_filename(&fname)
                                    .add_filter("Autosplitter Configuration", &["asc"])
                                    .add_filter("Any file", &["*"])
                                    .show_save_single_file()?;
                                let path = match path {
                                    Some(path) => path,
                                    None => return Ok(()),
                                };
                                let f = std::fs::OpenOptions::new()
                                    .create(true)
                                    .write(true)
                                    .truncate(true)
                                    .open(path)?;
                                serde_json::to_writer(&f, &*self.settings.read())?;
                                Ok(())
                            });
                        }
                    }
                    frame.quit();
                }
            });
        settings_editor
            .open(&mut self.show_settings_editor)
            .resizable(true)
            .collapsible(false)
            .hscroll(true)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.move_to_top(ui.layer_id());
                let mut settings = self.settings.write();
                let mut roots = settings.roots();
                show_children(&mut settings, ui, ctx, &mut roots);
            });
    }
}

#[allow(dead_code)]
fn customize_layout(layout: &mut LayoutSettings) {
    layout.components.iter_mut().for_each(customize_component);
}

#[allow(dead_code)]
fn customize_component(component: &mut ComponentSettings) {
    match component {
        ComponentSettings::Splits(splits) => customize_splits(splits),
        ComponentSettings::Timer(timer) => customize_timer(timer),
        _ => (),
    }
}

#[allow(dead_code)]
fn customize_splits(splits: &mut livesplit_core::component::splits::Settings) {
    use livesplit_core::timing::formatter::Accuracy;
    splits.visual_split_count = 5;
    splits.split_preview_count = 2;
    splits.split_time_accuracy = Accuracy::Tenths;
    splits.segment_time_accuracy = Accuracy::Tenths;
    splits.always_show_last_split = true;
    splits.delta_drop_decimals = true;
}

#[allow(dead_code)]
fn customize_timer(timer: &mut livesplit_core::component::timer::Settings) {
    use livesplit_core::timing::formatter::Accuracy;
    timer.accuracy = Accuracy::Tenths;
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    let polling_rate = 20.0;
    let frame_rate = 30.0;
    let settings = Settings::new();
    let settings = Arc::new(RwLock::new(settings));
    let mut run = Run::default();
    run.push_segment(Segment::new(""));

    let timer = Timer::new(run)
        .expect("Run with at least one segment provided")
        .into_shared();
    let options = eframe::NativeOptions {
        //always_on_top: true,
        // TODO: fix me
        initial_window_size: Some(egui::vec2(470.0, 337.0)),
        ..eframe::NativeOptions::default()
    };
    println!("size = {:#?}", options.initial_window_size);
    let latency = Arc::new(RwLock::new((0.0, 0.0)));

    let layout_settings = Layout::default_layout().settings();
    //customize_layout(&mut layout_settings);
    let layout = Layout::from_settings(layout_settings);

    let app = LiveSplitCoreRenderer {
        texture: None,
        timer: timer.clone(),
        layout,
        renderer: livesplit_core::rendering::software::Renderer::new(),
        show_settings_editor: false,
        settings: settings.clone(),
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
                        if devices.is_empty() {
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
                        let summary = snes.fetch_all(&mut client, &settings.read())?;
                        if summary.start {
                            timer.write().start();
                        }
                        if summary.reset {
                            // For now let the user manually reset
                            //timer.write().reset(true);
                            //snes = SNESState::new();
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
