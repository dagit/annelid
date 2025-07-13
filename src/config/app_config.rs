use clap::Parser;
use livesplit_hotkey::{Hotkey, KeyCode, Modifiers};
use serde_derive::{Deserialize, Serialize};

use crate::utils::*;
// use crate::hotkey::*;

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
    pub use_autosplitter: Option<bool>,
    #[clap(name = "polling-rate", long, short = 'p', value_parser)]
    pub polling_rate: Option<f32>,
    #[clap(name = "frame-rate", long, short = 'f', value_parser)]
    pub frame_rate: Option<f32>,
    #[clap(name = "reset-timer-on-game-reset", long, value_parser)]
    pub reset_timer_on_game_reset: Option<bool>,
    #[clap(name = "reset-game-on-timer-reset", long, value_parser)]
    pub reset_game_on_timer_reset: Option<bool>,
    #[clap(name = "global-hotkeys", long, short = 'g', value_parser)]
    pub global_hotkeys: Option<bool>,
    #[clap(skip)]
    pub hot_key_start: Option<Hotkey>,
    #[clap(skip)]
    pub hot_key_reset: Option<Hotkey>,
    #[clap(skip)]
    pub hot_key_undo: Option<Hotkey>,
    #[clap(skip)]
    pub hot_key_skip: Option<Hotkey>,
    #[clap(skip)]
    pub hot_key_pause: Option<Hotkey>,
    #[clap(skip)]
    pub hot_key_comparison_next: Option<Hotkey>,
    #[clap(skip)]
    pub hot_key_comparison_prev: Option<Hotkey>,
    #[clap(skip)]
    pub hot_key_toggle_global_hotkeys: Option<Hotkey>,
}

// #[derive(clap::ValueEnum, Clone, Copy, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
// pub enum YesOrNo {
//     #[default]
//     Yes,
//     No,
// }

pub const DEFAULT_FRAME_RATE: f32 = 30.0;
pub const DEFAULT_POLLING_RATE: f32 = 20.0;

impl AppConfig {
    pub fn new() -> Self {
        AppConfig {
            recent_splits: None,
            recent_layout: None,
            recent_autosplitter: None,
            hot_key_start: Some(Hotkey {
                key_code: KeyCode::Numpad1,
                modifiers: Modifiers::empty(),
            }),
            hot_key_reset: Some(Hotkey {
                key_code: KeyCode::Numpad3,
                modifiers: Modifiers::empty(),
            }),
            hot_key_undo: Some(Hotkey {
                key_code: KeyCode::Numpad8,
                modifiers: Modifiers::empty(),
            }),
            hot_key_skip: Some(Hotkey {
                key_code: KeyCode::Numpad2,
                modifiers: Modifiers::empty(),
            }),
            hot_key_pause: Some(Hotkey {
                key_code: KeyCode::Numpad5,
                modifiers: Modifiers::empty(),
            }),
            hot_key_comparison_next: Some(Hotkey {
                key_code: KeyCode::Numpad6,
                modifiers: Modifiers::empty(),
            }),
            hot_key_comparison_prev: Some(Hotkey {
                key_code: KeyCode::Numpad4,
                modifiers: Modifiers::empty(),
            }),
            hot_key_toggle_global_hotkeys: Some(Hotkey {
                key_code: KeyCode::Numpad9,
                modifiers: Modifiers::empty(),
            }),
            use_autosplitter: Some(false),
            frame_rate: Some(DEFAULT_FRAME_RATE),
            polling_rate: Some(DEFAULT_POLLING_RATE),
            reset_timer_on_game_reset: Some(false),
            reset_game_on_timer_reset: Some(false),
            global_hotkeys: Some(true),
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
