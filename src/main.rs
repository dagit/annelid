#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release mode
#[macro_use]
extern crate lazy_static;
pub mod autosplitters;
pub mod config;
pub mod hotkey;
pub mod livesplit_renderer;
pub mod routes;
pub mod ui;
pub mod usb2snes;
pub mod utils;
pub mod widget;

use autosplitters::supermetroid::Settings;
use clap::Parser;
use eframe::egui;
use livesplit_core::layout::{ComponentSettings, LayoutSettings};
use livesplit_core::{Layout, Run, Segment, Timer};
use parking_lot::RwLock;
use std::error::Error;
use std::sync::Arc;

use config::app_config::*;
use config::layout_meta::LayoutMeta;
use livesplit_renderer::*;

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
    #[cfg(feature = "tracing")]
    let _guard = {
        use tracing_chrome::ChromeLayerBuilder;
        use tracing_subscriber::prelude::*;

        let (chrome_layer, guard) = ChromeLayerBuilder::new().build();
        tracing_subscriber::registry().with(chrome_layer).init();
        guard
    };

    let cli_config = AppConfig::parse();
    let settings = Settings::new();
    let settings = Arc::new(RwLock::new(settings));
    let mut run = Run::default();
    run.push_segment(Segment::new(""));

    let timer = Timer::new(run)
        .expect("Run with at least one segment provided")
        .into_shared();

    let project_dirs = directories::ProjectDirs::from("", "", "annelid")
        .ok_or("Unable to computer configuration directory")?;
    println!("project_dirs = {project_dirs:#?}");

    let preference_dir = project_dirs.preference_dir();
    std::fs::create_dir_all(preference_dir)?;

    // Read saved config early so we can set viewport properties that must
    // be known before window creation (layout size/position, transparency).
    let saved_config: Option<AppConfig> = {
        let mut config_path = project_dirs.preference_dir().to_path_buf();
        config_path.push("settings.toml");
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
    };

    let layout_meta: Option<LayoutMeta> = (|| {
        let layout_path = cli_config
            .recent_layout
            .clone()
            .or_else(|| saved_config.as_ref()?.recent_layout.clone())?;
        LayoutMeta::from_layout_file(std::path::Path::new(&layout_path))
    })();

    let transparent = cli_config.transparent_window == Some(YesOrNo::Yes)
        || (cli_config.transparent_window.is_none()
            && saved_config.as_ref().and_then(|c| c.transparent_window) == Some(YesOrNo::Yes));

    let mut viewport = egui::viewport::ViewportBuilder::default();
    if transparent {
        viewport = viewport.with_transparent(true);
    }
    if let Some(ref meta) = layout_meta {
        viewport = meta.apply_to_viewport_builder(viewport);
    }

    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport,
        ..eframe::NativeOptions::default()
    };
    let layout_settings = Layout::default_layout(livesplit_core::Lang::English).settings();
    //customize_layout(&mut layout_settings);
    let layout = Layout::from_settings(layout_settings);

    use std::sync::mpsc::sync_channel;
    let (sync_sender, sync_receiver) = sync_channel(1);

    let mut app = LiveSplitCoreRenderer::new(
        timer,
        layout,
        settings,
        sync_sender,
        project_dirs,
        cli_config,
    );

    eframe::run_native(
        "Annelid",
        options,
        Box::new(move |cc| {
            livesplit_renderer::app_init(&mut app, sync_receiver, cc);
            Ok(Box::new(app))
        }),
    )?;
    Ok(())
}
