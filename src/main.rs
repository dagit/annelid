#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release mode
#[macro_use]
extern crate lazy_static;
extern crate gtk;
pub mod appconfig;
pub mod autosplitters;
pub mod gtk_ui;
pub mod livesplit;
pub mod routes;
pub mod usb2snes;
pub mod utils;

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
        frame_buffer: vec![],
        timer,
        layout,
        renderer: livesplit_core::rendering::software::BorrowedRenderer::new(),
        layout_state: None,
        show_settings_editor: false,
        settings,
        can_exit: false,
        is_exiting: false,
        thread_chan: sync_sender,
        project_dirs,
        app_config: Arc::new(parking_lot::lock_api::RwLock::new(cli_config)),
        app_config_processed: false,
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

    gtk_ui::main(app, frame_rate, polling_rate, latency, sync_receiver)
}
