#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release mode
#[macro_use]
extern crate lazy_static;
pub mod autosplitters;
pub mod routes;
pub mod usb2snes;

use autosplitters::supermetroid::{SNESState, Settings};
use clap::Parser;
use eframe::egui;
use livesplit_core::layout::{ComponentSettings, LayoutSettings};
use livesplit_core::{Layout, Run, Segment, SharedTimer, Timer};
use livesplit_hotkey::Hook;
use memoffset::offset_of;
use parking_lot::RwLock;
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use thread_priority::{set_current_thread_priority, ThreadBuilder, ThreadPriority};

fn messagebox_on_error<F>(f: F)
where
    F: FnOnce() -> std::result::Result<(), Box<dyn Error>>,
{
    use rfd::{MessageDialog, MessageLevel};
    match f() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", e);
            MessageDialog::new()
                .set_level(MessageLevel::Error)
                .set_title("Error")
                .set_description(format!("{}", e))
                .show();
        }
    }
}

fn print_on_error<F>(f: F)
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

#[derive(Deserialize, Serialize, Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct AppConfig {
    #[clap(name = "load-splits", short = 's', long, value_parser)]
    recent_splits: Option<String>,
    #[clap(name = "load-layout", short = 'l', long, value_parser)]
    recent_layout: Option<String>,
    #[clap(name = "load-autosplitter", short = 'a', long, value_parser)]
    recent_autosplitter: Option<String>,
    #[clap(name = "use-autosplitter", long, action)]
    use_autosplitter: Option<YesOrNo>,
    #[clap(name = "polling-rate", long, short = 'p', value_parser)]
    polling_rate: Option<f32>,
    #[clap(name = "frame-rate", long, short = 'f', value_parser)]
    frame_rate: Option<f32>,
    #[clap(name = "reset-timer-on-game-reset", long, short = 'r', value_parser)]
    reset_timer_on_game_reset: Option<YesOrNo>,
    #[clap(name = "reset-game-on-timer-reset", long, value_parser)]
    reset_game_on_timer_reset: Option<YesOrNo>,
    #[clap(name = "global-hotkeys", long, short = 'g', value_parser)]
    global_hotkeys: Option<YesOrNo>,
    #[clap(skip)]
    hot_key_start: Option<HotKey>,
    #[clap(skip)]
    hot_key_reset: Option<HotKey>,
    #[clap(skip)]
    hot_key_undo: Option<HotKey>,
    #[clap(skip)]
    hot_key_skip: Option<HotKey>,
    #[clap(skip)]
    hot_key_pause: Option<HotKey>,
    #[clap(skip)]
    hot_key_comparison_next: Option<HotKey>,
    #[clap(skip)]
    hot_key_comparison_prev: Option<HotKey>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
enum YesOrNo {
    #[default]
    Yes,
    No,
}

const DEFAULT_FRAME_RATE: f32 = 30.0;
const DEFAULT_POLLING_RATE: f32 = 20.0;

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

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
struct HotKey {
    key: ::egui::Key,
    modifiers: ::egui::Modifiers,
}

impl HotKey {
    fn to_livesplit_hotkey(self) -> livesplit_hotkey::Hotkey {
        to_livesplit_keycode(&self.key).with_modifiers(to_livesplit_modifiers(&self.modifiers))
    }
}

fn to_livesplit_keycode(key: &::egui::Key) -> livesplit_hotkey::KeyCode {
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
        egui::Key::F1 => F1,
        egui::Key::F2 => F2,
        egui::Key::F3 => F3,
        egui::Key::F4 => F4,
        egui::Key::F5 => F5,
        egui::Key::F6 => F6,
        egui::Key::F7 => F7,
        egui::Key::F8 => F8,
        egui::Key::F9 => F9,
        egui::Key::F10 => F10,
        egui::Key::F11 => F11,
        egui::Key::F12 => F12,
        egui::Key::F13 => F13,
        egui::Key::F14 => F14,
        egui::Key::F15 => F15,
        egui::Key::F16 => F16,
        egui::Key::F17 => F17,
        egui::Key::F18 => F18,
        egui::Key::F19 => F19,
        egui::Key::F20 => F20,
        egui::Key::Minus => Minus,
        egui::Key::PlusEquals => Equal,
    }
}

fn to_livesplit_keycode_alternative(key: &::egui::Key) -> Option<livesplit_hotkey::KeyCode> {
    use livesplit_hotkey::KeyCode::*;

    match key {
        egui::Key::Num0 => Some(Digit0),
        egui::Key::Num1 => Some(Digit1),
        egui::Key::Num2 => Some(Digit2),
        egui::Key::Num3 => Some(Digit3),
        egui::Key::Num4 => Some(Digit4),
        egui::Key::Num5 => Some(Digit5),
        egui::Key::Num6 => Some(Digit6),
        egui::Key::Num7 => Some(Digit7),
        egui::Key::Num8 => Some(Digit8),
        egui::Key::Num9 => Some(Digit9),
        _ => None,
    }
}

fn to_livesplit_modifiers(modifiers: &::egui::Modifiers) -> livesplit_hotkey::Modifiers {
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

struct LiveSplitCoreRenderer {
    frame_buffer: std::sync::Arc<std::sync::RwLock<Vec<u8>>>,
    layout: Layout,
    renderer: livesplit_core::rendering::software::BorrowedRenderer,
    layout_state: Option<livesplit_core::layout::LayoutState>,
    timer: SharedTimer,
    show_settings_editor: bool,
    settings: Arc<RwLock<Settings>>,
    can_exit: bool,
    is_exiting: bool,
    thread_chan: std::sync::mpsc::SyncSender<ThreadEvent>,
    project_dirs: directories::ProjectDirs,
    app_config: AppConfig,
    app_config_processed: bool,
    opengl_resources: std::sync::Arc<std::sync::RwLock<Option<OpenGLResources>>>,
    global_hotkey_hook: Option<Hook>,
}

fn from_de_error(e: toml::de::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
}

fn show_children(
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

impl LiveSplitCoreRenderer {
    fn confirm_save(&mut self, gl: &std::rc::Rc<glow::Context>) {
        use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
        let empty_path = "".to_owned();
        let document_dir = match directories::UserDirs::new() {
            None => empty_path,
            Some(d) => match d.document_dir() {
                None => empty_path,
                Some(d) => d.to_str().unwrap_or("").to_owned(),
            },
        };
        // TODO: fix this unwrap
        if self.timer.read().unwrap().run().has_been_modified() {
            let save_requested = MessageDialog::new()
                .set_level(MessageLevel::Warning)
                .set_title("Save Splits")
                .set_description("Splits have been modified. Save splits?")
                .set_buttons(MessageButtons::YesNo)
                .show();
            if save_requested == MessageDialogResult::Yes {
                self.save_splits_dialog(&document_dir);
            }
        }
        if self.settings.read().has_been_modified() {
            let save_requested = MessageDialog::new()
                .set_level(MessageLevel::Warning)
                .set_title("Save Autosplitter Config")
                .set_description(
                    "Autosplit config may have been modified. Save autosplitter config?",
                )
                .set_buttons(MessageButtons::YesNo)
                .show();
            if save_requested == MessageDialogResult::Yes {
                self.save_autosplitter_dialog(&document_dir);
            }
        }
        self.can_exit = true;
        unsafe {
            if let Some(opengl) = &*self.opengl_resources.read().unwrap() {
                use glow::HasContext;
                // TODO: is this everything we're supposed to delete?
                gl.delete_texture(opengl.texture);
                debug_assert!(gl.get_error() == 0, "1");
                gl.delete_buffer(opengl.vbo);
                debug_assert!(gl.get_error() == 0, "1");
                gl.delete_buffer(opengl.element_array_buffer);
                debug_assert!(gl.get_error() == 0, "1");
                gl.delete_vertex_array(opengl.vao);
                debug_assert!(gl.get_error() == 0, "1");
                gl.delete_program(opengl.program);
                debug_assert!(gl.get_error() == 0, "1");
                //self.opengl_resources = std::sync::Arc::new(std::sync::RwLock::new(None));
            }
        }
    }

    fn save_app_config(&self) {
        messagebox_on_error(|| {
            use std::io::Write;
            let mut config_path = self.project_dirs.preference_dir().to_path_buf();
            config_path.push("settings.toml");
            println!("Saving to {:#?}", config_path);
            let f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(config_path)?;
            let mut writer = std::io::BufWriter::new(f);
            let toml = toml::to_string_pretty(&self.app_config)?;
            writer.write_all(toml.as_bytes())?;
            writer.flush()?;
            Ok(())
        });
    }

    fn load_app_config(&mut self) {
        messagebox_on_error(|| {
            use std::io::Read;
            let mut config_path = self.project_dirs.preference_dir().to_path_buf();
            config_path.push("settings.toml");
            println!("Loading from {:#?}", config_path);
            let saved_config: AppConfig = std::fs::File::open(config_path)
                .and_then(|mut f| {
                    let mut buffer = String::new();
                    f.read_to_string(&mut buffer)?;
                    match toml::from_str(&buffer) {
                        Ok(app_config) => Ok(app_config),
                        Err(e) => Err(from_de_error(e)),
                    }
                })
                .unwrap_or_default();
            // Let the CLI options take precedent if any provided
            // TODO: this logic is bad, I really need to know if the CLI
            // stuff was present and whether the stuff was present in the config
            // but instead I just see two different states that need to be merged.
            let cli_config = self.app_config.clone();
            self.app_config = saved_config;
            if cli_config.recent_layout.is_some() {
                self.app_config.recent_layout = cli_config.recent_layout;
            }
            if cli_config.recent_splits.is_some() {
                self.app_config.recent_splits = cli_config.recent_splits;
            }
            if cli_config.recent_autosplitter.is_some() {
                self.app_config.recent_autosplitter = cli_config.recent_autosplitter;
            }
            if cli_config.use_autosplitter.is_some() {
                self.app_config.use_autosplitter = cli_config.use_autosplitter;
            }
            if cli_config.frame_rate.is_some() {
                self.app_config.frame_rate = cli_config.frame_rate;
            }
            if cli_config.polling_rate.is_some() {
                self.app_config.polling_rate = cli_config.polling_rate;
            }
            if cli_config.reset_timer_on_game_reset.is_some() {
                self.app_config.reset_timer_on_game_reset = cli_config.reset_timer_on_game_reset;
            }
            if cli_config.reset_game_on_timer_reset.is_some() {
                self.app_config.reset_game_on_timer_reset = cli_config.reset_game_on_timer_reset;
            }
            if cli_config.global_hotkeys.is_some() {
                self.app_config.global_hotkeys = cli_config.global_hotkeys;
            }
            Ok(())
        });
    }

    fn process_app_config(&mut self, ctx: &egui::Context) {
        messagebox_on_error(|| {
            // Now that we've converged on a config, try loading what we can
            if let Some(layout) = &self.app_config.recent_layout {
                let f = std::fs::File::open(layout)?;
                self.load_layout(f, ctx)?;
            }
            if let Some(splits) = &self.app_config.recent_splits {
                let f = std::fs::File::open(splits)?;
                let path = std::path::Path::new(splits)
                    .parent()
                    .ok_or("failed to find parent directory")?;
                self.load_splits(f, path.to_path_buf())?;
            }
            if let Some(autosplitter) = &self.app_config.recent_autosplitter {
                let f = std::fs::File::open(autosplitter)?;
                self.load_autosplitter(f)?;
            }
            Ok(())
        });
    }

    fn load_layout(&mut self, f: std::fs::File, ctx: &egui::Context) -> Result<(), Box<dyn Error>> {
        use std::io::Read;
        let mut reader = std::io::BufReader::new(f);
        let mut layout_file = String::new();
        reader.read_to_string(&mut layout_file)?;

        self.layout = livesplit_core::layout::parser::parse(&layout_file)?;
        let doc = roxmltree::Document::parse(&layout_file)?;
        doc.root().children().for_each(|d| {
            if d.tag_name().name() == "Layout" {
                use std::str::FromStr;
                let mut mode = None;
                let mut x = None;
                let mut y = None;
                let mut width = None;
                let mut height = None;
                d.children().for_each(|d| {
                    if d.tag_name().name() == "Mode" {
                        mode = d.text();
                    }
                    if d.tag_name().name() == "X" {
                        x = d.text().and_then(|d| f32::from_str(d).ok());
                    }
                    if d.tag_name().name() == "Y" {
                        y = d.text().and_then(|d| f32::from_str(d).ok());
                    }
                    if mode.is_some() && d.tag_name().name() == format!("{}Width", mode.unwrap()) {
                        width = d.text().and_then(|d| f32::from_str(d).ok());
                    }
                    if mode.is_some() && d.tag_name().name() == format!("{}Height", mode.unwrap()) {
                        height = d.text().and_then(|d| f32::from_str(d).ok());
                    }
                    if let (Some(x), Some(y), Some(width), Some(height)) = (x, y, width, height) {
                        ctx.send_viewport_cmd(egui::viewport::ViewportCommand::InnerSize(
                            egui::Vec2::new(width, height),
                        ));
                        ctx.send_viewport_cmd(egui::viewport::ViewportCommand::OuterPosition(
                            egui::Pos2::new(x, y),
                        ));
                    }
                });
            }
        });
        Ok(())
    }

    fn load_splits(
        &mut self,
        f: std::fs::File,
        path: std::path::PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        use livesplit_core::run::parser::composite;
        use std::io::Read;
        let file_contents: Result<Vec<_>, _> = f.bytes().collect();
        // TODO: fix this unwrap
        *self.timer.write().unwrap() =
            Timer::new(composite::parse(&file_contents?, path.parent())?.run)?;
        Ok(())
    }

    fn load_autosplitter(&mut self, f: std::fs::File) -> Result<(), Box<dyn Error>> {
        *self.settings.write() = serde_json::from_reader(std::io::BufReader::new(f))?;
        Ok(())
    }

    fn save_splits_dialog(&mut self, default_dir: &str) {
        // TODO: fix this unwrap
        let mut fname = self.timer.read().unwrap().run().extended_file_name(false);
        let splits = self.app_config.recent_splits.as_ref().unwrap_or_else(|| {
            if fname.is_empty() {
                fname += "annelid.lss";
            } else {
                fname += ".lss";
            }
            &fname
        });
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .recent_splits
            .as_ref()
            .map_or(default_path_buf.clone(), |p| {
                let path = std::path::Path::new(&p);
                path.parent().map_or(default_path_buf, |p| p.to_path_buf())
            })
            .into_os_string()
            .into_string()
            .expect("utf8");
        self.save_dialog(
            &dir,
            &splits.clone(),
            ("LiveSplit Splits", "lss"),
            |me, f| {
                use livesplit_core::run::saver::livesplit::IoWrite;
                let writer = IoWrite(&f);
                // TODO: fix this unwrap
                livesplit_core::run::saver::livesplit::save_timer(
                    &me.timer.read().unwrap(),
                    writer,
                )?;
                Ok(())
            },
        );
    }

    fn save_autosplitter_dialog(&mut self, default_dir: &str) {
        // TODO: fix this unwrap
        let mut fname = self.timer.read().unwrap().run().extended_file_name(false);
        let autosplitter = self
            .app_config
            .recent_autosplitter
            .as_ref()
            .unwrap_or_else(|| {
                if fname.is_empty() {
                    fname += "annelid.asc";
                } else {
                    fname += ".asc";
                }
                &fname
            });
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .recent_autosplitter
            .as_ref()
            .map_or(default_path_buf.clone(), |p| {
                let path = std::path::Path::new(&p);
                path.parent().map_or(default_path_buf, |p| p.to_path_buf())
            })
            .into_os_string()
            .into_string()
            .expect("utf8");
        self.save_dialog(
            &dir,
            &autosplitter.clone(),
            ("Autosplitter Configuration", "asc"),
            |me, f| {
                serde_json::to_writer(&f, &*me.settings.read())?;
                Ok(())
            },
        );
    }

    fn save_dialog(
        &mut self,
        default_dir: &str,
        default_fname: &str,
        file_type: (&str, &str),
        save_action: impl FnOnce(&mut Self, std::fs::File) -> Result<(), Box<dyn Error>>,
    ) {
        use rfd::FileDialog;
        messagebox_on_error(|| {
            let path = FileDialog::new()
                .set_directory(default_dir)
                .set_file_name(default_fname)
                .add_filter(file_type.0, &[file_type.1])
                .add_filter("Any file", &["*"])
                .save_file();
            let path = match path {
                Some(path) => path,
                None => return Ok(()),
            };
            let f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?;
            save_action(self, f)?;
            Ok(())
        });
    }

    fn open_layout_dialog(&mut self, default_dir: &str, ctx: &egui::Context) {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .recent_layout
            .as_ref()
            .map_or(default_path_buf.clone(), |p| {
                let path = std::path::Path::new(&p);
                path.parent().map_or(default_path_buf, |p| p.to_path_buf())
            })
            .into_os_string()
            .into_string()
            .expect("utf8");
        self.open_dialog(&dir, ("LiveSplit Layout", "lsl"), |me, f, path| {
            me.load_layout(f, ctx)?;
            me.app_config.recent_layout = Some(path.into_os_string().into_string().expect("utf8"));
            Ok(())
        });
    }

    fn open_splits_dialog(&mut self, default_dir: &str) {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .recent_splits
            .as_ref()
            .map_or(default_path_buf.clone(), |p| {
                let path = std::path::Path::new(&p);
                path.parent().map_or(default_path_buf, |p| p.to_path_buf())
            })
            .into_os_string()
            .into_string()
            .expect("utf8");
        self.open_dialog(&dir, ("LiveSplit Splits", "lss"), |me, f, path| {
            me.load_splits(f, path.clone())?;
            me.app_config.recent_splits = Some(path.into_os_string().into_string().expect("utf8"));
            Ok(())
        });
    }

    fn open_autosplitter_dialog(&mut self, default_dir: &str) {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .recent_autosplitter
            .as_ref()
            .map_or(default_path_buf.clone(), |p| {
                let path = std::path::Path::new(&p);
                path.parent().map_or(default_path_buf, |p| p.to_path_buf())
            })
            .into_os_string()
            .into_string()
            .expect("utf8");
        self.open_dialog(
            &dir,
            ("Autosplitter Configuration", "asc"),
            |me, f, path| {
                me.load_autosplitter(f)?;
                me.app_config.recent_autosplitter =
                    Some(path.into_os_string().into_string().expect("utf8"));
                Ok(())
            },
        );
    }

    fn open_dialog(
        &mut self,
        default_dir: &str,
        file_type: (&str, &str),
        open_action: impl FnOnce(
            &mut Self,
            std::fs::File,
            std::path::PathBuf,
        ) -> Result<(), Box<dyn Error>>,
    ) {
        use rfd::FileDialog;
        messagebox_on_error(|| {
            let path = FileDialog::new()
                .set_directory(default_dir)
                .add_filter(file_type.0, &[file_type.1])
                .add_filter("Any file", &["*"])
                .pick_file();
            let path = match path {
                Some(path) => path,
                None => return Ok(()),
            };
            let f = std::fs::File::open(path.clone())?;
            open_action(self, f, path)?;
            Ok(())
        });
    }

    fn enable_global_hotkeys(&mut self) -> Result<(), Box<dyn Error>> {
        // It would be more elegant to use get_or_insert_with, however
        // the `with` branch cannot have a `Result` type if we do that.
        let hook: &Hook = match self.global_hotkey_hook.as_ref() {
            None => {
                self.global_hotkey_hook = Some(Hook::new()?);
                self.global_hotkey_hook.as_ref().unwrap()
            }
            Some(h) => h,
        };
        print!("Registering global hotkeys...");
        // TODO: this is kind of gross because of the logical duplication
        // between egui input handling and global hotkey handling
        // Work is needed to keep them in sync :(
        let timer = self.timer.clone();
        if let Some(hot_key) = self.app_config.hot_key_start {
            hook.register(hot_key.to_livesplit_hotkey(), move || {
                // TODO: fix this unwrap
                timer.write().unwrap().split_or_start();
            })?;
            if let Some(alt_key) = to_livesplit_keycode_alternative(&hot_key.key) {
                let alternative = livesplit_hotkey::Hotkey {
                    key_code: alt_key,
                    modifiers: to_livesplit_modifiers(&hot_key.modifiers),
                };
                let timer = self.timer.clone();
                hook.register(alternative, move || {
                    // TODO: fix this unwrap
                    timer.write().unwrap().split_or_start();
                })?;
            }
        }
        let timer = self.timer.clone();
        // TODO: this is not ideal because if the app_config or thread_chan
        // change after this function is called, these will point to the old
        // values. Probably need to wrap config and thread_chan in Arc
        let config = self.app_config.clone();
        let thread_chan = self.thread_chan.clone();
        if let Some(hot_key) = self.app_config.hot_key_reset {
            hook.register(hot_key.to_livesplit_hotkey(), move || {
                // TODO: fix this unwrap
                timer.write().unwrap().reset(true);
                if config.use_autosplitter == Some(YesOrNo::Yes) {
                    thread_chan.try_send(ThreadEvent::TimerReset).unwrap_or(());
                }
            })?;
            if let Some(alt_key) = to_livesplit_keycode_alternative(&hot_key.key) {
                let alternative = livesplit_hotkey::Hotkey {
                    key_code: alt_key,
                    modifiers: to_livesplit_modifiers(&hot_key.modifiers),
                };
                let timer = self.timer.clone();
                let thread_chan = self.thread_chan.clone();
                hook.register(alternative, move || {
                    // TODO: fix this unwrap
                    timer.write().unwrap().reset(true);
                    if config.use_autosplitter == Some(YesOrNo::Yes) {
                        thread_chan.try_send(ThreadEvent::TimerReset).unwrap_or(());
                    }
                })?;
            }
        }
        let timer = self.timer.clone();
        if let Some(hot_key) = self.app_config.hot_key_undo {
            hook.register(hot_key.to_livesplit_hotkey(), move || {
                // TODO: fix this unwrap
                timer.write().unwrap().undo_split();
            })?;
            if let Some(alt_key) = to_livesplit_keycode_alternative(&hot_key.key) {
                let alternative = livesplit_hotkey::Hotkey {
                    key_code: alt_key,
                    modifiers: to_livesplit_modifiers(&hot_key.modifiers),
                };
                let timer = self.timer.clone();
                hook.register(alternative, move || {
                    // TODO: fix this unwrap
                    timer.write().unwrap().undo_split();
                })?;
            }
        }
        let timer = self.timer.clone();
        if let Some(hot_key) = self.app_config.hot_key_skip {
            hook.register(hot_key.to_livesplit_hotkey(), move || {
                // TODO: fix this unwrap
                timer.write().unwrap().skip_split();
            })?;
            if let Some(alt_key) = to_livesplit_keycode_alternative(&hot_key.key) {
                let alternative = livesplit_hotkey::Hotkey {
                    key_code: alt_key,
                    modifiers: to_livesplit_modifiers(&hot_key.modifiers),
                };
                let timer = self.timer.clone();
                hook.register(alternative, move || {
                    // TODO: fix this unwrap
                    timer.write().unwrap().skip_split();
                })?;
            }
        }
        let timer = self.timer.clone();
        if let Some(hot_key) = self.app_config.hot_key_pause {
            hook.register(hot_key.to_livesplit_hotkey(), move || {
                // TODO: fix this unwrap
                timer.write().unwrap().toggle_pause();
            })?;
            if let Some(alt_key) = to_livesplit_keycode_alternative(&hot_key.key) {
                let alternative = livesplit_hotkey::Hotkey {
                    key_code: alt_key,
                    modifiers: to_livesplit_modifiers(&hot_key.modifiers),
                };
                let timer = self.timer.clone();
                hook.register(alternative, move || {
                    // TODO: fix this unwrap
                    timer.write().unwrap().toggle_pause();
                })?;
            }
        }
        let timer = self.timer.clone();
        if let Some(hot_key) = self.app_config.hot_key_comparison_next {
            hook.register(hot_key.to_livesplit_hotkey(), move || {
                // TODO: fix this unwrap
                timer.write().unwrap().switch_to_next_comparison();
            })?;
            if let Some(alt_key) = to_livesplit_keycode_alternative(&hot_key.key) {
                let alternative = livesplit_hotkey::Hotkey {
                    key_code: alt_key,
                    modifiers: to_livesplit_modifiers(&hot_key.modifiers),
                };
                let timer = self.timer.clone();
                hook.register(alternative, move || {
                    // TODO: fix this unwrap
                    timer.write().unwrap().switch_to_next_comparison();
                })?;
            }
        }
        let timer = self.timer.clone();
        if let Some(hot_key) = self.app_config.hot_key_comparison_prev {
            hook.register(hot_key.to_livesplit_hotkey(), move || {
                // TODO: fix this unwrap
                timer.write().unwrap().switch_to_previous_comparison();
            })?;
            if let Some(alt_key) = to_livesplit_keycode_alternative(&hot_key.key) {
                let alternative = livesplit_hotkey::Hotkey {
                    key_code: alt_key,
                    modifiers: to_livesplit_modifiers(&hot_key.modifiers),
                };
                let timer = self.timer.clone();
                hook.register(alternative, move || {
                    // TODO: fix this unwrap
                    timer.write().unwrap().switch_to_previous_comparison();
                })?;
            }
        }
        println!("registered");
        Ok(())
    }
}

#[derive(Debug)]
struct OpenGLResources {
    program: glow::Program,
    u_screen_size: glow::UniformLocation,
    u_sampler: glow::UniformLocation,
    vbo: glow::Buffer,
    vao: glow::VertexArray,
    element_array_buffer: glow::Buffer,
    texture: glow::Texture,
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
#[repr(C)]
struct Vertex {
    pos: epaint::Pos2,
    uv: epaint::Pos2,
}

impl eframe::App for LiveSplitCoreRenderer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        //let update_timer = std::time::Instant::now();
        if !self.app_config_processed {
            self.process_app_config(ctx);
            self.app_config_processed = true;
            // Since this block should only run once, we abuse it to also
            // set a thread priority only once. We want rendering to take a
            // relative backseat to anything else the user has going on
            // like an emulator.
            set_current_thread_priority(ThreadPriority::Min).unwrap_or(())
        }
        let viewport = ctx.input(|i| i.screen_rect);
        {
            let timer = self.timer.read().unwrap();
            let snapshot = timer.snapshot();
            // a local scope so the timer lock has a smaller scope
            // TODO: fix this unwrap
            match &mut self.layout_state {
                None => {
                    self.layout_state = Some(self.layout.state(&snapshot));
                }
                Some(layout_state) => {
                    self.layout.update_state(layout_state, &snapshot);
                }
            };
        }
        let sz = viewport.size();
        let w = viewport.max.x - viewport.min.x;
        let h = viewport.max.y - viewport.min.y;

        if let Some(layout_state) = &self.layout_state {
            let szu32 = [sz.x as u32, sz.y as u32];
            let sz = [sz.x as usize, sz.y as usize];
            self.frame_buffer
                .write()
                .unwrap()
                .resize(sz[0] * sz[1] * 4, 0);
            self.renderer.render(
                layout_state,
                self.frame_buffer.write().unwrap().as_mut_slice(),
                szu32,
                sz[0] as u32,
                false,
            );
        }
        let gl_ctx = if self.opengl_resources.read().unwrap().is_some() {
            self.opengl_resources.clone()
        } else {
            unsafe {
                use eframe::glow::HasContext;
                let gl = frame.gl().unwrap();
                let vert = gl.create_shader(glow::VERTEX_SHADER).expect("create vert");
                debug_assert_eq!(gl.get_error(), 0);
                let source = "
#version 330

uniform vec2 u_screen_size;
in      vec2 a_pos;
in      vec2 a_tc;
out     vec2 v_tc;

void main() {
    gl_Position = vec4(
                      2.0 * a_pos.x / u_screen_size.x - 1.0,
                      1.0 - 2.0 * a_pos.y / u_screen_size.y,
                      0.0,
                      1.0);
    v_tc = a_tc;
}";
                debug_assert_eq!(gl.get_error(), 0);
                gl.shader_source(vert, source);
                debug_assert_eq!(gl.get_error(), 0);
                gl.compile_shader(vert);
                debug_assert_eq!(gl.get_error(), 0);
                debug_assert!(
                    gl.get_shader_compile_status(vert),
                    "{}",
                    gl.get_shader_info_log(vert)
                );
                debug_assert_eq!(gl.get_error(), 0);

                let frag = gl
                    .create_shader(glow::FRAGMENT_SHADER)
                    .expect("crate fragment");
                let source = "
#version 330

uniform sampler2D u_sampler;

in      vec2      v_tc;

out     vec4      fragmentColor;

void main() {
    fragmentColor = texture(u_sampler, v_tc);
}
";
                debug_assert_eq!(gl.get_error(), 0);
                gl.shader_source(frag, source);
                debug_assert_eq!(gl.get_error(), 0);
                gl.compile_shader(frag);
                debug_assert_eq!(gl.get_error(), 0);
                debug_assert!(
                    gl.get_shader_compile_status(frag),
                    "{}",
                    gl.get_shader_info_log(frag)
                );
                debug_assert_eq!(gl.get_error(), 0);
                let program = gl.create_program().expect("create program");
                debug_assert_eq!(gl.get_error(), 0);
                gl.attach_shader(program, vert);
                debug_assert_eq!(gl.get_error(), 0);
                gl.attach_shader(program, frag);
                debug_assert_eq!(gl.get_error(), 0);
                gl.link_program(program);
                debug_assert_eq!(gl.get_error(), 0);
                debug_assert!(gl.get_program_link_status(program), "link failed");
                debug_assert_eq!(gl.get_error(), 0);
                gl.detach_shader(program, vert);
                debug_assert_eq!(gl.get_error(), 0);
                gl.detach_shader(program, frag);
                debug_assert_eq!(gl.get_error(), 0);
                gl.delete_shader(vert);
                debug_assert_eq!(gl.get_error(), 0);
                gl.delete_shader(frag);
                debug_assert_eq!(gl.get_error(), 0);
                let u_screen_size = gl.get_uniform_location(program, "u_screen_size").unwrap();
                debug_assert_eq!(gl.get_error(), 0);
                let u_sampler = gl.get_uniform_location(program, "u_sampler").unwrap();
                debug_assert_eq!(gl.get_error(), 0);

                let vbo = gl.create_buffer().expect("vbo creation");
                debug_assert_eq!(gl.get_error(), 0);

                let a_pos_loc = gl.get_attrib_location(program, "a_pos").unwrap();
                debug_assert_eq!(gl.get_error(), 0);
                let a_tc_loc = gl.get_attrib_location(program, "a_tc").unwrap();
                debug_assert_eq!(gl.get_error(), 0);

                let stride = std::mem::size_of::<Vertex>() as i32;
                let vao = gl.create_vertex_array().unwrap();
                debug_assert_eq!(gl.get_error(), 0);
                gl.bind_vertex_array(Some(vao));
                debug_assert_eq!(gl.get_error(), 0);
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                debug_assert_eq!(gl.get_error(), 0);
                gl.vertex_attrib_pointer_f32(
                    a_pos_loc,
                    2,
                    glow::FLOAT,
                    false,
                    stride,
                    offset_of!(Vertex, pos) as i32,
                );
                debug_assert_eq!(gl.get_error(), 0);
                gl.vertex_attrib_pointer_f32(
                    a_tc_loc,
                    2,
                    glow::FLOAT,
                    false,
                    stride,
                    offset_of!(Vertex, uv) as i32,
                );
                debug_assert_eq!(gl.get_error(), 0);

                let element_array_buffer = gl.create_buffer().expect("create element_array_buffer");
                debug_assert_eq!(gl.get_error(), 0);
                let texture = gl.create_texture().expect("create texture");
                debug_assert_eq!(gl.get_error(), 0);
                let resources = OpenGLResources {
                    element_array_buffer,
                    program,
                    texture,
                    u_sampler,
                    u_screen_size,
                    vao,
                    vbo,
                };
                //println!("{:?}", resources);
                std::sync::Arc::new(std::sync::RwLock::new(Some(resources)))
            }
        };

        let frame_buffer = self.frame_buffer.clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            let callback = egui::PaintCallback {
                rect: viewport,
                callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                    unsafe {
                        use eframe::glow::HasContext;
                        let ctx = gl_ctx.read().unwrap();
                        let ctx = ctx.as_ref().unwrap();

                        //let timer = std::time::Instant::now();
                        //let gl = frame.gl().expect("Rendering context");
                        let gl = painter.gl();
                        gl.use_program(Some(ctx.program));
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.bind_vertex_array(Some(ctx.vao));
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.enable_vertex_attrib_array(0);
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.enable_vertex_attrib_array(1);
                        debug_assert_eq!(gl.get_error(), 0);

                        gl.uniform_2_f32(Some(&ctx.u_screen_size), w, h);
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.uniform_1_i32(Some(&ctx.u_sampler), 0);
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.active_texture(glow::TEXTURE0);
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.bind_vertex_array(Some(ctx.vao));
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ctx.element_array_buffer));
                        debug_assert_eq!(gl.get_error(), 0);

                        debug_assert_eq!(gl.get_error(), 0);
                        gl.bind_texture(glow::TEXTURE_2D, Some(ctx.texture));
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.tex_parameter_i32(
                            glow::TEXTURE_2D,
                            glow::TEXTURE_MAG_FILTER,
                            glow::NEAREST as _,
                        );
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.tex_parameter_i32(
                            glow::TEXTURE_2D,
                            glow::TEXTURE_MIN_FILTER,
                            glow::NEAREST as _,
                        );
                        debug_assert_eq!(gl.get_error(), 0);

                        gl.tex_parameter_i32(
                            glow::TEXTURE_2D,
                            glow::TEXTURE_WRAP_S,
                            glow::CLAMP_TO_EDGE as i32,
                        );
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.tex_parameter_i32(
                            glow::TEXTURE_2D,
                            glow::TEXTURE_WRAP_T,
                            glow::CLAMP_TO_EDGE as i32,
                        );
                        debug_assert_eq!(gl.get_error(), 0);

                        gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
                        debug_assert_eq!(gl.get_error(), 0);
                        //println!("({},{})", w, h);
                        gl.tex_image_2d(
                            glow::TEXTURE_2D,
                            0,
                            glow::RGBA8 as _,
                            w as _,
                            h as _,
                            0,
                            glow::RGBA,
                            glow::UNSIGNED_BYTE,
                            Some(frame_buffer.read().unwrap().as_slice()),
                        );
                        debug_assert_eq!(gl.get_error(), 0);

                        use epaint::Pos2;
                        let vertices: Vec<Vertex> = vec![
                            Vertex {
                                // top right
                                pos: Pos2::new(viewport.max.x, viewport.min.y),
                                uv: Pos2::new(1.0, 0.0),
                            },
                            Vertex {
                                // bottom right
                                pos: Pos2::new(viewport.max.x, viewport.max.y),
                                uv: Pos2::new(1.0, 1.0),
                            },
                            Vertex {
                                // bottom left
                                pos: Pos2::new(viewport.min.x, viewport.max.y),
                                uv: Pos2::new(0.0, 1.0),
                            },
                            Vertex {
                                // top left
                                pos: Pos2::new(viewport.min.x, viewport.min.y),
                                uv: Pos2::new(0.0, 0.0),
                            },
                        ];
                        //println!("{:#?}", vertices);
                        let indices: Vec<u32> = vec![
                            // note that we start from 0!
                            0, 1, 3, // first triangle
                            1, 2, 3, // second triangle
                        ];

                        debug_assert_eq!(gl.get_error(), 0);
                        gl.bind_buffer(glow::ARRAY_BUFFER, Some(ctx.vbo));
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.buffer_data_u8_slice(
                            glow::ARRAY_BUFFER,
                            bytemuck::cast_slice(vertices.as_slice()),
                            glow::STREAM_DRAW,
                        );
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ctx.element_array_buffer));
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.buffer_data_u8_slice(
                            glow::ELEMENT_ARRAY_BUFFER,
                            bytemuck::cast_slice(indices.as_slice()),
                            glow::STREAM_DRAW,
                        );
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.bind_texture(glow::TEXTURE_2D, Some(ctx.texture));
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.draw_elements(
                            glow::TRIANGLES,
                            indices.len() as i32,
                            glow::UNSIGNED_INT,
                            0,
                        );
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.disable_vertex_attrib_array(0);
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.disable_vertex_attrib_array(1);
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.bind_vertex_array(None);
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.bind_buffer(glow::ARRAY_BUFFER, None);
                        debug_assert_eq!(gl.get_error(), 0);
                        gl.use_program(None);
                        debug_assert_eq!(gl.get_error(), 0);
                        //println!("Time to render texture: {}μs", timer.elapsed().as_micros());
                    }
                })),
            };
            ui.painter().add(callback);
        });
        ctx.set_visuals(egui::Visuals::dark()); // Switch to dark mode
        let settings_editor = egui::containers::Window::new("Settings Editor");
        egui::Area::new("livesplit")
            .enabled(!self.show_settings_editor)
            .show(ctx, |ui| {
                ui.set_width(ctx.input(|i| i.screen_rect.width()));
                ui.set_height(ctx.input(|i| i.screen_rect.height()));
            })
            .response
            .context_menu(|ui| {
                let empty_path = "".to_owned();
                let document_dir = match directories::UserDirs::new() {
                    None => empty_path,
                    Some(d) => match d.document_dir() {
                        None => empty_path,
                        Some(d) => d.to_str().unwrap_or("").to_owned(),
                    },
                };
                ui.menu_button("LiveSplit Save/Load", |ui| {
                    if ui.button("Import Layout").clicked() {
                        ui.close_menu();
                        self.open_layout_dialog(&document_dir, ctx);
                    }
                    if ui.button("Import Splits").clicked() {
                        ui.close_menu();
                        self.open_splits_dialog(&document_dir);
                    }
                    if ui.button("Save Splits as...").clicked() {
                        ui.close_menu();
                        self.save_splits_dialog(&document_dir);
                    }
                });
                ui.menu_button("Run Control", |ui| {
                    if ui.button("Start").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().start();
                        ui.close_menu()
                    }
                    if ui.button("Split").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().split();
                        ui.close_menu()
                    }
                    ui.separator();
                    if ui.button("Skip Split").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().skip_split();
                        ui.close_menu()
                    }
                    if ui.button("Undo Split").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().undo_split();
                        ui.close_menu()
                    }
                    ui.separator();
                    if ui.button("Pause").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().pause();
                        ui.close_menu()
                    }

                    if ui.button("Resume").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().resume();
                        ui.close_menu()
                    }
                    ui.separator();
                    if ui.button("Reset").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().reset(true);
                        if self.app_config.use_autosplitter == Some(YesOrNo::Yes) {
                            self.thread_chan
                                .try_send(ThreadEvent::TimerReset)
                                .unwrap_or(());
                        }
                        ui.close_menu()
                    }
                });
                ui.menu_button("Autosplitter", |ui| {
                    if ui.button("Configure").clicked() {
                        self.show_settings_editor = true;
                        ui.close_menu();
                    }
                    if ui.button("Load Configuration").clicked() {
                        ui.close_menu();
                        self.open_autosplitter_dialog(&document_dir);
                    }
                    if ui.button("Save Configuration").clicked() {
                        ui.close_menu();
                        self.save_autosplitter_dialog(&document_dir);
                    }
                });
                ui.separator();
                ui.add(
                    egui::widgets::Label::new(format!(
                        "Comparison: {}",
                        self.timer.read().unwrap().current_comparison()
                    ))
                    .wrap(false),
                );
                ui.separator();
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Close)
                }
            });
        settings_editor
            .open(&mut self.show_settings_editor)
            .resizable(true)
            .collapsible(false)
            .hscroll(true)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.move_to_top(ui.layer_id());
                let mut settings = self.settings.write();
                let mut roots = settings.roots();
                show_children(&mut settings, ui, ctx, &mut roots);
            });
        ctx.input(|i| {
            i.events.iter().for_each(|e| {
                if let egui::Event::Scroll(v) = e {
                    if v.y > 0.0 {
                        self.layout.scroll_up();
                    } else {
                        self.layout.scroll_down();
                    }
                }
            })
        });
        if self.app_config.global_hotkeys != Some(YesOrNo::Yes) {
            ctx.input_mut(|input| {
                if let Some(hot_key) = self.app_config.hot_key_start {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().split_or_start();
                    }
                }
                if let Some(hot_key) = self.app_config.hot_key_reset {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().reset(true);
                        if self.app_config.use_autosplitter == Some(YesOrNo::Yes) {
                            self.thread_chan
                                .try_send(ThreadEvent::TimerReset)
                                .unwrap_or(());
                        }
                    }
                }
                if let Some(hot_key) = self.app_config.hot_key_undo {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().undo_split();
                    }
                }
                if let Some(hot_key) = self.app_config.hot_key_skip {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().skip_split();
                    }
                }
                if let Some(hot_key) = self.app_config.hot_key_pause {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().toggle_pause();
                    }
                }
                if let Some(hot_key) = self.app_config.hot_key_comparison_next {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().switch_to_next_comparison();
                    }
                }
                if let Some(hot_key) = self.app_config.hot_key_comparison_prev {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().switch_to_previous_comparison();
                    }
                }
            });
        }

        ctx.input(|i| {
            if i.viewport().close_requested() {
                self.is_exiting = true;
                self.confirm_save(frame.gl().expect("No GL context"));
                self.save_app_config();
            }
        });
        if self.can_exit {
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Close);
        } else {
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::CancelClose)
        }
        //println!("Time to update: {}μs", update_timer.elapsed().as_micros());
    }
}

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

enum ThreadEvent {
    TimerReset,
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    let cli_config = AppConfig::parse();
    let settings = Settings::new();
    let settings = Arc::new(RwLock::new(settings));
    let mut run = Run::default();
    run.push_segment(Segment::new(""));

    let timer = Timer::new(run)
        .expect("Run with at least one segment provided")
        .into_shared();
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::viewport::ViewportBuilder {
            ..Default::default()
        },
        ..eframe::NativeOptions::default()
    };
    let latency = Arc::new(RwLock::new((0.0, 0.0)));

    let layout_settings = Layout::default_layout().settings();
    //customize_layout(&mut layout_settings);
    let layout = Layout::from_settings(layout_settings);

    use std::sync::mpsc::sync_channel;
    let (sync_sender, sync_receiver) = sync_channel(1);

    let project_dirs = directories::ProjectDirs::from("", "", "annelid")
        .ok_or("Unable to computer configuration directory")?;
    println!("project_dirs = {:#?}", project_dirs);

    let preference_dir = project_dirs.preference_dir();
    std::fs::create_dir_all(preference_dir)?;

    let mut app = LiveSplitCoreRenderer {
        frame_buffer: std::sync::Arc::new(std::sync::RwLock::new(vec![])),
        timer: timer.clone(),
        layout,
        renderer: livesplit_core::rendering::software::BorrowedRenderer::new(),
        layout_state: None,
        show_settings_editor: false,
        settings: settings.clone(),
        can_exit: false,
        is_exiting: false,
        thread_chan: sync_sender,
        project_dirs,
        app_config: cli_config,
        app_config_processed: false,
        opengl_resources: std::sync::Arc::new(std::sync::RwLock::new(None)),
        global_hotkey_hook: None,
    };

    eframe::run_native(
        "Annelid",
        options,
        Box::new(move |cc| {
            let context = cc.egui_ctx.clone();
            app.load_app_config();
            if app.app_config.global_hotkeys == Some(YesOrNo::Yes) {
                messagebox_on_error(|| app.enable_global_hotkeys());
            }
            let frame_rate = app.app_config.frame_rate.unwrap_or(DEFAULT_FRAME_RATE);
            let polling_rate = app.app_config.polling_rate.unwrap_or(DEFAULT_POLLING_RATE);
            // This thread is essentially just a refresh rate timer
            // it ensures that the gui thread is redrawn at the requested frame_rate,
            // possibly more often.
            let _frame_rate_thread = ThreadBuilder::default()
                .name("Frame Rate Thread".to_owned())
                .priority(ThreadPriority::Min)
                .spawn(move |_| loop {
                    context.request_repaint();
                    std::thread::sleep(std::time::Duration::from_millis(
                        (1000.0 / frame_rate) as u64,
                    ));
                })
                // TODO: fix this unwrap
                .unwrap();
            // This thread deals with polling the SNES at a fixed rate.
            if app.app_config.use_autosplitter == Some(YesOrNo::Yes) {
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
                                if summary.reset
                                    && app.app_config.reset_timer_on_game_reset
                                        == Some(YesOrNo::Yes)
                                {
                                    // TODO: fix this unwrap
                                    timer.write().unwrap().reset(true);
                                }
                                if summary.split {
                                    timer
                                        .write()
                                        .unwrap()
                                        .set_game_time(snes.gametime_to_seconds());
                                    // TODO: fix this unwrap
                                    timer.write().unwrap().split();
                                }
                                {
                                    *latency.write() =
                                        (summary.latency_average, summary.latency_stddev);
                                }
                                // If the timer gets reset, we need to make a fresh snes state
                                if let Ok(ThreadEvent::TimerReset) = sync_receiver.try_recv() {
                                    snes = SNESState::new();
                                    //Reset the snes
                                    if app.app_config.reset_game_on_timer_reset
                                        == Some(YesOrNo::Yes)
                                    {
                                        client.reset()?;
                                    }
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

            Box::new(app)
        }),
    )?;
    Ok(())
}
