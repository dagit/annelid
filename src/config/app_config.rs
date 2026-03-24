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
    #[clap(name = "renderer", long, short = 'r', value_parser)]
    pub renderer: Option<RendererType>,
    #[clap(name = "diag-mode", long, value_parser)]
    pub diag_mode: Option<DiagMode>,
    #[clap(skip)]
    pub transparent_window: Option<YesOrNo>,
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

#[derive(clap::ValueEnum, Clone, Copy, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum RendererType {
    Software,
    #[default]
    Gpu,
}

/// Diagnostic modes for debugging rendering issues (ghosting, etc.).
/// Use via CLI: `annelid --diag-mode flash-frames`
#[derive(clap::ValueEnum, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagMode {
    /// Flash alternating red/blue frames to detect double-buffer issues
    FlashFrames,
    /// Skip ALL glClear calls to see if clearing is helping or hurting
    NoClear,
    /// Disable blending globally to test for blend state leaks
    NoBlend,
    /// Force triple-clear at every stage of the pipeline
    TripleClear,
    /// Software renderer only: disable PBO double-buffering
    SinglePbo,
    /// Log every GL clear/draw operation per frame (verbose)
    LogOps,
}

pub const DEFAULT_FRAME_RATE: f32 = 30.0;
pub const DEFAULT_POLLING_RATE: f32 = 20.0;

impl AppConfig {
    fn new() -> Self {
        use crate::hotkey::{KeyCode, Modifiers};
        let modifiers = Modifiers::default();
        AppConfig {
            recent_splits: None,
            recent_layout: None,
            recent_autosplitter: None,
            hot_key_start: Some(HotKey {
                key: KeyCode::Num1,
                modifiers,
            }),
            hot_key_reset: Some(HotKey {
                key: KeyCode::Num3,
                modifiers,
            }),
            hot_key_undo: Some(HotKey {
                key: KeyCode::Num8,
                modifiers,
            }),
            hot_key_skip: Some(HotKey {
                key: KeyCode::Num2,
                modifiers,
            }),
            hot_key_pause: Some(HotKey {
                key: KeyCode::Num5,
                modifiers,
            }),
            hot_key_comparison_next: Some(HotKey {
                key: KeyCode::Num6,
                modifiers,
            }),
            hot_key_comparison_prev: Some(HotKey {
                key: KeyCode::Num4,
                modifiers,
            }),
            use_autosplitter: Some(YesOrNo::Yes),
            frame_rate: Some(DEFAULT_FRAME_RATE),
            polling_rate: Some(DEFAULT_POLLING_RATE),
            reset_timer_on_game_reset: Some(YesOrNo::No),
            reset_game_on_timer_reset: Some(YesOrNo::No),
            global_hotkeys: Some(YesOrNo::Yes),
            renderer: Some(RendererType::Gpu),
            diag_mode: None,
            transparent_window: None,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig::new()
    }
}
