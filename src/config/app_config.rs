use crate::autosplitters;
use clap::Parser;
use serde_derive::{Deserialize, Serialize};

use crate::hotkey::*;
use crate::utils::*;

#[derive(Default, Deserialize, Serialize, Parser, Debug, Clone)]
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
    #[clap(skip)]
    pub autosplitter_type: Option<autosplitters::AType>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum YesOrNo {
    #[default]
    Yes,
    No,
}

impl From<bool> for YesOrNo {
    fn from(b: bool) -> Self {
        match b {
            true => YesOrNo::Yes,
            false => YesOrNo::No,
        }
    }
}

impl From<YesOrNo> for bool {
    fn from(yes: YesOrNo) -> Self {
        match yes {
            YesOrNo::Yes => true,
            YesOrNo::No => false,
        }
    }
}

pub const DEFAULT_FRAME_RATE: f32 = 30.0;
pub const DEFAULT_POLLING_RATE: f32 = 20.0;

impl AppConfig {
    pub fn new() -> Self {
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
            autosplitter_type: Some(autosplitters::AType::QUSB2SNES),
        }
    }

    pub fn save_app_config(&self) {
        use std::io::Write;
        let project_dirs = directories::ProjectDirs::from("", "", "annelid") // get directories
            .ok_or("Unable to load computer configuration directory");
        println!("project_dirs = {project_dirs:#?}");

        let config_dir = project_dirs.unwrap(); // get preferences directory
        println!("project_dirs = {:#?}", config_dir.preference_dir());

        messagebox_on_error(|| {
            std::fs::create_dir_all(config_dir.preference_dir()).expect("Created config dir"); // create preferences directory

            let mut config_path = config_dir.preference_dir().to_path_buf();
            config_path.push("settings.toml");

            println!("Saving to {config_path:#?}");
            let f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(config_path)?;
            let mut writer = std::io::BufWriter::new(f);
            let toml = toml::to_string_pretty(&self)?;
            writer.write_all(toml.as_bytes())?;
            writer.flush()?;
            Ok(())
        });
    }

    pub fn load_app_config(mut self) -> Self {
        let project_dirs = directories::ProjectDirs::from("", "", "annelid") // get directories
            .ok_or("Unable to load computer configuration directory");
        println!("project_dirs = {project_dirs:#?}");

        let config_dir = project_dirs.unwrap(); // get preferences directory
        println!("project_dirs = {:#?}", config_dir.preference_dir());

        messagebox_on_error(|| {
            use std::io::Read;
            let mut config_path = config_dir.preference_dir().to_path_buf();
            config_path.push("settings.toml");
            println!("Loading from {config_path:#?}");
            let saved_config: AppConfig = std::fs::File::open(config_path)
                .and_then(|mut f| {
                    let mut buffer = String::new();
                    f.read_to_string(&mut buffer)?;
                    match toml::from_str(&buffer) {
                        Ok(app_config) => Ok(app_config),
                        Err(e) => Err(from_de_error(e)),
                    }

                    // }).unwrap;
                })
                .unwrap_or_default();
            self = saved_config;
            Ok(())
        });
        self
    }
}

// impl Default for AppConfig {
//     fn default() -> Self {
//         AppConfig::new()
//     }
// }
