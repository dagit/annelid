use crate::appconfig::{AppConfig, YesOrNo};
use crate::autosplitters::supermetroid::{SNESState, Settings};
use crate::livesplit::*;
use crate::usb2snes;
#[cfg(not(windows))]
use eframe::egui;
use livesplit_core::layout::{ComponentSettings, LayoutSettings};
use livesplit_core::SharedTimer;
use parking_lot::RwLock;
use std::error::Error;
use std::sync::Arc;
use thread_priority::{ThreadBuilder, ThreadPriority};

pub fn messagebox_on_error<F>(f: F)
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

pub fn print_on_error<F>(f: F)
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

#[cfg(not(windows))]
pub fn show_children(
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

// These are no longer used really. They were handy for testing early on
// and at this point they could be removed.
pub fn customize_layout(layout: &mut LayoutSettings) {
    layout.components.iter_mut().for_each(customize_component);
}

pub fn customize_component(component: &mut ComponentSettings) {
    match component {
        ComponentSettings::Splits(splits) => customize_splits(splits),
        ComponentSettings::Timer(timer) => customize_timer(timer),
        _ => (),
    }
}

pub fn customize_splits(splits: &mut livesplit_core::component::splits::Settings) {
    use livesplit_core::timing::formatter::Accuracy;
    splits.visual_split_count = 5;
    splits.split_preview_count = 2;
    splits.split_time_accuracy = Accuracy::Tenths;
    splits.segment_time_accuracy = Accuracy::Tenths;
    splits.always_show_last_split = true;
    splits.delta_drop_decimals = true;
}

pub fn customize_timer(timer: &mut livesplit_core::component::timer::Settings) {
    use livesplit_core::timing::formatter::Accuracy;
    timer.accuracy = Accuracy::Tenths;
}

#[cfg(windows)]
pub type RepaintHandle = Arc<RwLock<windows::Win32::Foundation::HWND>>;

#[cfg(not(windows))]
pub type RepaintHandle = egui::Context;

// TODO: move this to a method in livesplit
pub fn repaint_timer(frame_rate: f32, handle: RepaintHandle) {
    // This thread is essentially just a refresh rate timer
    // it ensures that the gui thread is redrawn at the requested frame_rate,
    // possibly more often.
    let _frame_rate_thread = ThreadBuilder::default()
        .name("Frame Rate Thread".to_owned())
        .priority(ThreadPriority::Min)
        .spawn(move |_| loop {
            #[cfg(not(windows))]
            handle.request_repaint();
            #[cfg(windows)]
            unsafe {
                use windows::Win32::Graphics::Gdi::InvalidateRect;
                let h = { *handle.read() };
                if h != windows::Win32::Foundation::HWND(0) {
                    //println!("sending repaint");
                    InvalidateRect(h, None, false);
                }
            };
            std::thread::sleep(std::time::Duration::from_millis(
                (1000.0 / frame_rate) as u64,
            ));
        })
        // TODO: fix this unwrap
        .unwrap();
}

// TODO: it would probably be cleaner to make this thread a method of LiveSplitCoreRenderer
pub fn snes_polling(
    app_config: Arc<RwLock<AppConfig>>,
    polling_rate: f32,
    latency: Arc<RwLock<(f32, f32)>>,
    timer: SharedTimer,
    settings: Arc<RwLock<Settings>>,
    sync_receiver: std::sync::mpsc::Receiver<ThreadEvent>,
) {
    let _snes_polling_thread = ThreadBuilder::default()
        .name("SNES Polling Thread".to_owned())
        // We could change this thread priority, but we probably
        // should leave it at the default to make sure we get timely
        // polling of SNES state
        .spawn(move |_| loop {
            print_on_error(|| -> std::result::Result<(), Box<dyn Error>> {
                let mut client = usb2snes::SyncClient::connect()?;
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
                        // TODO: fix this unwrap
                        timer.write().unwrap().start();
                    }
                    if summary.reset && app_config.read().reset_on_reset == Some(YesOrNo::Yes) {
                        // TODO: fix this unwrap
                        timer.write().unwrap().reset(true);
                        snes = SNESState::new();
                    }
                    if summary.split {
                        // TODO: fix this unwrap
                        timer.write().unwrap().split();
                    }
                    {
                        *latency.write() = (summary.latency_average, summary.latency_stddev);
                    }
                    // If the timer gets reset, we need to make a fresh snes state
                    if let Ok(ThreadEvent::TimerReset) = sync_receiver.try_recv() {
                        snes = SNESState::new();
                    }
                    std::thread::sleep(std::time::Duration::from_millis(
                        (1000.0 / polling_rate) as u64,
                    ));
                }
            });
            std::thread::sleep(std::time::Duration::from_millis(1000));
        })
        //TODO: fix this unwrap
        .unwrap();
}
