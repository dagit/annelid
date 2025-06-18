#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release mode
#[macro_use]
extern crate lazy_static;
pub mod autosplitters;
pub mod config;
pub mod hotkey;
pub mod livesplit_renderer;
pub mod routes;
pub mod usb2snes;
pub mod utils;
pub mod widget;

use autosplitters::supermetroid::Settings;
use clap::Parser;
use eframe::egui;
use livesplit_core::layout::{ComponentSettings, LayoutSettings};
use livesplit_core::{Layout, Run, Segment, Timer};
use parking_lot::RwLock;
use std::env;
use std::error::Error;
use std::sync::Arc;

use config::app_config::*;
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
    let mut config = AppConfig::load_app_config(AppConfig::new()); // load saved config into default object
    AppConfig::update_from(&mut config, env::args()); // reads command line options into app config object
    
    let settings = Settings::new(); // setttings for SM autosplitter
    let settings = Arc::new(RwLock::new(settings)); // creates a RW pointer to the autosplitter settings
    
    let mut run = Run::default(); // creates a default livesplit run object
    run.push_segment(Segment::new("")); // push blank segment to run

    let timer = Timer::new(run) // create timer object
        .expect("Run with at least one segment provided") // error message
        .into_shared(); // makes timer sharable across threads
    
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::viewport::ViewportBuilder {
            ..Default::default()
        },
        ..eframe::NativeOptions::default()
    }; // create egui display options
    
    let layout_settings = Layout::default_layout().settings(); // create default layout settings
    //customize_layout(&mut layout_settings);
    let layout = Layout::from_settings(layout_settings); // create default layout

    use std::sync::mpsc::sync_channel;
    let (sync_sender, sync_receiver) = sync_channel(1); // create thread

    let mut app = LiveSplitCoreRenderer::new(
        timer,
        layout,
        settings,
        sync_sender,
        // project_dirs,
        config,
    ); // create livesplit-core renderer object

    eframe::run_native(
        "Annelid",
        options,
        Box::new(move |cc| {
            livesplit_renderer::app_init(&mut app, sync_receiver, cc);
            Ok(Box::new(app))
        }), // initialize livesplitrender and load into egui render box
    )?;
    Ok(())
}
