#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release mode
#[macro_use]
extern crate lazy_static;
#[cfg(target_os = "linux")]
extern crate gtk;
pub mod appconfig;
pub mod autosplitters;
#[cfg(target_os = "linux")]
pub mod linux;
pub mod livesplit;
pub mod routes;
pub mod usb2snes;
pub mod utils;
#[cfg(windows)]
pub mod win32;

use appconfig::{AppConfig, YesOrNo};
use autosplitters::supermetroid::Settings;
use clap::Parser;
use livesplit::*;
use livesplit_core::{Layout, Run, Segment, Timer};
#[allow(unused_imports)]
use memoffset::offset_of;
use parking_lot::RwLock;
use std::error::Error;
use std::sync::Arc;
#[allow(unused_imports)]
use thread_priority::{set_current_thread_priority, ThreadBuilder, ThreadPriority};
use utils::*;

fn main() -> std::result::Result<(), Box<dyn Error>> {
    let cli_config = AppConfig::parse();
    let settings = Settings::new();
    let settings = Arc::new(RwLock::new(settings));
    let mut run = Run::default();
    run.push_segment(Segment::new(""));

    let timer = Timer::new(run)
        .expect("Run with at least one segment provided")
        .into_shared();
    #[cfg(not(windows))]
    let _options = eframe::NativeOptions {
        ..eframe::NativeOptions::default()
    };
    let latency = Arc::new(RwLock::new((0.0, 0.0)));

    let layout_settings = Layout::default_layout().settings();
    //customize_layout(&mut layout_settings);
    let layout = Layout::from_settings(layout_settings);

    use std::sync::mpsc::sync_channel;
    let (sync_sender, sync_receiver) = sync_channel(0);

    let project_dirs = directories::ProjectDirs::from("", "", "annelid")
        .ok_or("Unable to computer configuration directory")?;
    println!("project_dirs = {:#?}", project_dirs);

    let preference_dir = project_dirs.preference_dir();
    std::fs::create_dir_all(preference_dir)?;

    let mut app = LiveSplitCoreRenderer {
        #[cfg(not(windows))]
        frame_buffer: vec![],
        timer,
        layout,
        #[cfg(windows)]
        renderer: livesplit_core::rendering::software::Renderer::new(),
        #[cfg(not(windows))]
        renderer: livesplit_core::rendering::software::BorrowedRenderer::new(),
        #[cfg(not(windows))]
        layout_state: None,
        #[cfg(not(windows))]
        show_settings_editor: false,
        #[cfg(any(windows, target_os = "linux"))]
        settings,
        #[cfg(target_os = "macos")]
        settings: settings.clone(),
        can_exit: false,
        #[cfg(not(windows))]
        is_exiting: false,
        thread_chan: sync_sender,
        project_dirs,
        app_config: Arc::new(parking_lot::lock_api::RwLock::new(cli_config)),
        #[cfg(not(windows))]
        app_config_processed: false,
        #[cfg(not(windows))]
        opengl_resources: None,
        global_hotkey_hook: None,
    };

    app.load_app_config();
    if app.app_config.read().global_hotkeys == Some(YesOrNo::Yes) {
        messagebox_on_error(|| app.enable_global_hotkeys());
    }
    let frame_rate = app
        .app_config
        .read()
        .frame_rate
        .unwrap_or(appconfig::DEFAULT_FRAME_RATE);
    let polling_rate = app
        .app_config
        .read()
        .polling_rate
        .unwrap_or(appconfig::DEFAULT_POLLING_RATE);

    #[cfg(windows)]
    {
        let mut window = win32::main(app)?;
        window.run(frame_rate, polling_rate, latency, sync_receiver)?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        crate::linux::main(app, frame_rate, polling_rate, latency, sync_receiver)?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    eframe::run_native(
        "Annelid",
        _options,
        Box::new(move |cc| {
            let context = cc.egui_ctx.clone();
            repaint_timer(frame_rate, context);
            // This thread deals with polling the SNES at a fixed rate.
            if app.app_config.read().use_autosplitter == Some(YesOrNo::Yes) {
                snes_polling(
                    app.app_config.clone(),
                    polling_rate,
                    latency,
                    app.timer.clone(),
                    settings,
                    sync_receiver,
                );
            }

            Box::new(app)
        }),
    );
}
