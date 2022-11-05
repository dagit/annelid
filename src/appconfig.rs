use clap::Parser;
use eframe::egui;
use serde_derive::{Deserialize, Serialize};

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
    #[clap(name = "reset-timer-on-game-reset", long, short = 'r', value_parser)]
    pub reset_on_reset: Option<YesOrNo>,
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
            use_autosplitter: Some(YesOrNo::Yes),
            frame_rate: Some(DEFAULT_FRAME_RATE),
            polling_rate: Some(DEFAULT_POLLING_RATE),
            reset_on_reset: Some(YesOrNo::No),
            global_hotkeys: Some(YesOrNo::Yes),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig::new()
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub struct HotKey {
    pub key: ::egui::Key,
    pub modifiers: ::egui::Modifiers,
}

impl HotKey {
    pub fn to_livesplit_hotkey(self) -> livesplit_hotkey::Hotkey {
        to_livesplit_keycode(&self.key).with_modifiers(to_livesplit_modifiers(&self.modifiers))
    }
}

pub fn to_livesplit_keycode(key: &::egui::Key) -> livesplit_hotkey::KeyCode {
    use livesplit_hotkey::KeyCode::*;

    match key {
        egui::Key::ArrowDown => ArrowDown,
        egui::Key::ArrowLeft => ArrowLeft,
        egui::Key::ArrowRight => ArrowRight,
        egui::Key::ArrowUp => ArrowUp,
        egui::Key::Escape => Escape,
        egui::Key::Tab => Tab,
        egui::Key::Backspace => Backspace,
        egui::Key::Enter => Enter,
        egui::Key::Space => Space,
        egui::Key::Insert => Insert,
        egui::Key::Delete => Delete,
        egui::Key::Home => Home,
        egui::Key::End => End,
        egui::Key::PageUp => PageUp,
        egui::Key::PageDown => PageDown,
        egui::Key::Num0 => Numpad0,
        egui::Key::Num1 => Numpad1,
        egui::Key::Num2 => Numpad2,
        egui::Key::Num3 => Numpad3,
        egui::Key::Num4 => Numpad4,
        egui::Key::Num5 => Numpad5,
        egui::Key::Num6 => Numpad6,
        egui::Key::Num7 => Numpad7,
        egui::Key::Num8 => Numpad8,
        egui::Key::Num9 => Numpad9,
        egui::Key::A => KeyA,
        egui::Key::B => KeyB,
        egui::Key::C => KeyC,
        egui::Key::D => KeyD,
        egui::Key::E => KeyE,
        egui::Key::F => KeyF,
        egui::Key::G => KeyG,
        egui::Key::H => KeyH,
        egui::Key::I => KeyI,
        egui::Key::J => KeyJ,
        egui::Key::K => KeyK,
        egui::Key::L => KeyL,
        egui::Key::M => KeyM,
        egui::Key::N => KeyN,
        egui::Key::O => KeyO,
        egui::Key::P => KeyP,
        egui::Key::Q => KeyQ,
        egui::Key::R => KeyR,
        egui::Key::S => KeyS,
        egui::Key::T => KeyT,
        egui::Key::U => KeyU,
        egui::Key::V => KeyV,
        egui::Key::W => KeyW,
        egui::Key::X => KeyX,
        egui::Key::Y => KeyY,
        egui::Key::Z => KeyZ,
    }
}

pub fn to_livesplit_modifiers(modifiers: &::egui::Modifiers) -> livesplit_hotkey::Modifiers {
    use livesplit_hotkey::Modifiers;
    let mut mods = Modifiers::empty();
    if modifiers.shift {
        mods.insert(Modifiers::SHIFT)
    };
    if modifiers.ctrl {
        mods.insert(Modifiers::CONTROL)
    };
    if modifiers.alt {
        mods.insert(Modifiers::ALT)
    };
    if modifiers.mac_cmd || modifiers.command {
        mods.insert(Modifiers::META)
    };
    mods
}
