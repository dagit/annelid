use clap::Parser;
use serde_derive::{Deserialize, Serialize};

use crate::hotkey::*;

#[derive(Deserialize, Serialize, Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct AppConfig {
    #[clap(name = "load-splits", short = 's', long, value_parser)]
    pub recent_splits: Option<String>,
    #[clap(name = "load-layout", short = 'l', long, value_parser)]
    pub recent_layout: Option<String>,
    #[clap(name = "load-autosplitter", short = 'a', long, value_parser)]
    pub recent_autosplitter: Option<String>,
    #[clap(name = "use-autosplitter", long, action)]
    pub use_autosplitter: Option<YesOrNo>,
    #[clap(name = "polling-rate", long, short = 'p', value_parser)]
    pub polling_rate: Option<f32>,
    #[clap(name = "frame-rate", long, short = 'f', value_parser)]
    pub frame_rate: Option<f32>,
    #[clap(name = "reset-timer-on-game-reset", long, value_parser)]
    pub reset_timer_on_game_reset: Option<YesOrNo>,
    #[clap(name = "reset-game-on-timer-reset", long, value_parser)]
    pub reset_game_on_timer_reset: Option<YesOrNo>,
    #[clap(name = "global-hotkeys", long, short = 'g', value_parser)]
    pub global_hotkeys: Option<YesOrNo>,
    #[clap(skip)]
    pub hot_key_start: Option<HotKey>,
    #[clap(skip)]
    pub hot_key_reset: Option<HotKey>,
    #[clap(skip)]
    pub hot_key_undo: Option<HotKey>,
    #[clap(skip)]
    pub hot_key_skip: Option<HotKey>,
    #[clap(skip)]
    pub hot_key_pause: Option<HotKey>,
    #[clap(skip)]
    pub hot_key_comparison_next: Option<HotKey>,
    #[clap(skip)]
    pub hot_key_comparison_prev: Option<HotKey>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum YesOrNo {
    #[default]
    Yes,
    No,
}

pub const DEFAULT_FRAME_RATE: f32 = 30.0;
pub const DEFAULT_POLLING_RATE: f32 = 20.0;

impl AppConfig {
    fn new() -> Self {
        let modifiers = ::egui::Modifiers::default();
        AppConfig {
            recent_splits: None,
            recent_layout: None,
            recent_autosplitter: None,
            hot_key_start: Some(HotKey {
                key: egui::Key::Num1,
                modifiers,
            }),
            hot_key_reset: Some(HotKey {
                key: egui::Key::Num3,
                modifiers,
            }),
            hot_key_undo: Some(HotKey {
                key: egui::Key::Num8,
                modifiers,
            }),
            hot_key_skip: Some(HotKey {
                key: egui::Key::Num2,
                modifiers,
            }),
            hot_key_pause: Some(HotKey {
                key: egui::Key::Num5,
                modifiers,
            }),
            hot_key_comparison_next: Some(HotKey {
                key: egui::Key::Num6,
                modifiers,
            }),
            hot_key_comparison_prev: Some(HotKey {
                key: egui::Key::Num4,
                modifiers,
            }),
            use_autosplitter: Some(YesOrNo::Yes),
            frame_rate: Some(DEFAULT_FRAME_RATE),
            polling_rate: Some(DEFAULT_POLLING_RATE),
            reset_timer_on_game_reset: Some(YesOrNo::No),
            reset_game_on_timer_reset: Some(YesOrNo::No),
            global_hotkeys: Some(YesOrNo::Yes),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig::new()
    }
}
