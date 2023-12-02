use crate::autosplitters::supermetroid::{SNESState, Settings};
use eframe::egui;
use livesplit_core::{Layout, SharedTimer, Timer};
use livesplit_hotkey::Hook;
use parking_lot::RwLock;
use std::error::Error;
use std::sync::Arc;
use thread_priority::{set_current_thread_priority, ThreadBuilder, ThreadPriority};

use crate::config::app_config::*;
use crate::hotkey::*;
use crate::utils::*;
use crate::widget::glow_canvas::*;

pub enum ThreadEvent {
    TimerReset,
}

pub struct LiveSplitCoreRenderer {
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
    pub app_config: std::sync::Arc<std::sync::RwLock<AppConfig>>,
    app_config_processed: bool,
    glow_canvas: GlowCanvas,
    global_hotkey_hook: Option<Hook>,
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
    pub fn new(
        timer: SharedTimer,
        layout: Layout,
        settings: Arc<RwLock<Settings>>,
        chan: std::sync::mpsc::SyncSender<ThreadEvent>,
        project_dirs: directories::ProjectDirs,
        cli_config: AppConfig,
    ) -> Self {
        LiveSplitCoreRenderer {
            timer,
            layout,
            renderer: livesplit_core::rendering::software::BorrowedRenderer::new(),
            layout_state: None,
            show_settings_editor: false,
            settings,
            can_exit: false,
            is_exiting: false,
            thread_chan: chan,
            project_dirs,
            app_config: std::sync::Arc::new(std::sync::RwLock::new(cli_config)),
            app_config_processed: false,
            glow_canvas: GlowCanvas::new(),
            global_hotkey_hook: None,
        }
    }

    pub fn confirm_save(&mut self, gl: &std::rc::Rc<glow::Context>) {
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
        self.glow_canvas.destroy(gl);
    }

    pub fn save_app_config(&self) {
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

    pub fn load_app_config(&mut self) {
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
            let cli_config = self.app_config.read().unwrap().clone();
            let mut new_app_config = saved_config;
            if cli_config.recent_layout.is_some() {
                new_app_config.recent_layout = cli_config.recent_layout;
            }
            if cli_config.recent_splits.is_some() {
                new_app_config.recent_splits = cli_config.recent_splits;
            }
            if cli_config.recent_autosplitter.is_some() {
                new_app_config.recent_autosplitter = cli_config.recent_autosplitter;
            }
            if cli_config.use_autosplitter.is_some() {
                new_app_config.use_autosplitter = cli_config.use_autosplitter;
            }
            if cli_config.frame_rate.is_some() {
                new_app_config.frame_rate = cli_config.frame_rate;
            }
            if cli_config.polling_rate.is_some() {
                new_app_config.polling_rate = cli_config.polling_rate;
            }
            if cli_config.reset_timer_on_game_reset.is_some() {
                new_app_config.reset_timer_on_game_reset = cli_config.reset_timer_on_game_reset;
            }
            if cli_config.reset_game_on_timer_reset.is_some() {
                new_app_config.reset_game_on_timer_reset = cli_config.reset_game_on_timer_reset;
            }
            if cli_config.global_hotkeys.is_some() {
                new_app_config.global_hotkeys = cli_config.global_hotkeys;
            }
            *self.app_config.write().unwrap() = new_app_config;
            Ok(())
        });
    }

    pub fn process_app_config(&mut self, ctx: &egui::Context) {
        messagebox_on_error(|| {
            // Now that we've converged on a config, try loading what we can
            let config = self.app_config.read().unwrap().clone();
            if let Some(layout) = config.recent_layout {
                let f = std::fs::File::open(layout)?;
                self.load_layout(&f, ctx)?;
            }
            if let Some(splits) = config.recent_splits {
                let f = std::fs::File::open(&splits)?;
                let path = std::path::Path::new(&splits)
                    .parent()
                    .ok_or("failed to find parent directory")?;
                self.load_splits(&f, path.to_path_buf())?;
            }
            if let Some(autosplitter) = config.recent_autosplitter {
                let f = std::fs::File::open(autosplitter)?;
                self.load_autosplitter(&f)?;
            }
            Ok(())
        });
    }

    pub fn load_layout(
        &mut self,
        f: &std::fs::File,
        ctx: &egui::Context,
    ) -> Result<(), Box<dyn Error>> {
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

    pub fn load_splits(
        &mut self,
        f: &std::fs::File,
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

    pub fn load_autosplitter(&mut self, f: &std::fs::File) -> Result<(), Box<dyn Error>> {
        *self.settings.write() = serde_json::from_reader(std::io::BufReader::new(f))?;
        Ok(())
    }

    pub fn save_splits_dialog(&mut self, default_dir: &str) {
        // TODO: fix this unwrap
        let mut fname = self.timer.read().unwrap().run().extended_file_name(false);
        let splits = self
            .app_config
            .read()
            .unwrap()
            .recent_splits
            .clone()
            .unwrap_or_else(|| {
                if fname.is_empty() {
                    fname += "annelid.lss";
                } else {
                    fname += ".lss";
                }
                fname
            });
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .unwrap()
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

    pub fn save_autosplitter_dialog(&mut self, default_dir: &str) {
        // TODO: fix this unwrap
        let mut fname = self.timer.read().unwrap().run().extended_file_name(false);
        let autosplitter: String = self
            .app_config
            .read()
            .unwrap()
            .recent_autosplitter
            .clone()
            .unwrap_or_else(|| {
                if fname.is_empty() {
                    fname += "annelid.asc";
                } else {
                    fname += ".asc";
                }
                fname
            });
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .unwrap()
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

    pub fn save_dialog(
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

    pub fn open_layout_dialog(&mut self, default_dir: &str, ctx: &egui::Context) {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .unwrap()
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
            me.load_layout(&f, ctx)?;
            me.app_config.write().unwrap().recent_layout =
                Some(path.into_os_string().into_string().expect("utf8"));
            Ok(())
        });
    }

    pub fn open_splits_dialog(&mut self, default_dir: &str) {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .unwrap()
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
            me.load_splits(&f, path.clone())?;
            me.app_config.write().unwrap().recent_splits =
                Some(path.into_os_string().into_string().expect("utf8"));
            Ok(())
        });
    }

    pub fn open_autosplitter_dialog(&mut self, default_dir: &str) {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .unwrap()
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
                me.load_autosplitter(&f)?;
                me.app_config.write().unwrap().recent_autosplitter =
                    Some(path.into_os_string().into_string().expect("utf8"));
                Ok(())
            },
        );
    }

    pub fn open_dialog(
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

    pub fn enable_global_hotkeys(&mut self) -> Result<(), Box<dyn Error>> {
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
        if let Some(hot_key) = self.app_config.read().unwrap().hot_key_start {
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
        let app_config = self.app_config.clone();
        let thread_chan = self.thread_chan.clone();
        if let Some(hot_key) = self.app_config.read().unwrap().hot_key_reset {
            let app_config_ = app_config.clone();
            hook.register(hot_key.to_livesplit_hotkey(), move || {
                // TODO: fix this unwrap
                timer.write().unwrap().reset(true);
                if app_config_.read().unwrap().use_autosplitter == Some(YesOrNo::Yes) {
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
                    if app_config.read().unwrap().use_autosplitter == Some(YesOrNo::Yes) {
                        thread_chan.try_send(ThreadEvent::TimerReset).unwrap_or(());
                    }
                })?;
            }
        }
        let timer = self.timer.clone();
        if let Some(hot_key) = self.app_config.read().unwrap().hot_key_undo {
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
        if let Some(hot_key) = self.app_config.read().unwrap().hot_key_skip {
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
        if let Some(hot_key) = self.app_config.read().unwrap().hot_key_pause {
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
        if let Some(hot_key) = self.app_config.read().unwrap().hot_key_comparison_next {
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
        if let Some(hot_key) = self.app_config.read().unwrap().hot_key_comparison_prev {
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
        ctx.input(|i| {
            if i.viewport().close_requested() {
                self.is_exiting = true;
                self.confirm_save(frame.gl().expect("No GL context"));
                self.save_app_config();
            }
        });
        if self.can_exit {
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Close);
            return;
        } else {
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::CancelClose)
        }
        let viewport = ctx.input(|i| i.screen_rect);
        self.glow_canvas.update_frame_buffer(|frame_buffer| {
            {
                let timer = self.timer.read().unwrap();
                let snapshot = timer.snapshot();
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

            if let Some(layout_state) = &self.layout_state {
                let szu32 = [sz.x as u32, sz.y as u32];
                let sz = [sz.x as usize, sz.y as usize];
                {
                    let mut buffer = frame_buffer.lock().unwrap();
                    buffer.resize(sz[0] * sz[1] * 4, 0);
                    self.renderer.render(
                        layout_state,
                        buffer.as_mut_slice(),
                        szu32,
                        sz[0] as u32,
                        false,
                    );
                }
            }
        });
        self.glow_canvas
            .paint_layer(ctx, egui::LayerId::background(), viewport);
        //self.glow_canvas.paint_immediate(frame.gl().unwrap(), viewport);
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
                        if self.app_config.read().unwrap().use_autosplitter == Some(YesOrNo::Yes) {
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
        {
            let config = self.app_config.read().unwrap();
            if config.global_hotkeys != Some(YesOrNo::Yes) {
                ctx.input_mut(|input| {
                    if let Some(hot_key) = config.hot_key_start {
                        if input.consume_key(hot_key.modifiers, hot_key.key) {
                            // TODO: fix this unwrap
                            self.timer.write().unwrap().split_or_start();
                        }
                    }
                    if let Some(hot_key) = config.hot_key_reset {
                        if input.consume_key(hot_key.modifiers, hot_key.key) {
                            // TODO: fix this unwrap
                            self.timer.write().unwrap().reset(true);
                            if config.use_autosplitter == Some(YesOrNo::Yes) {
                                self.thread_chan
                                    .try_send(ThreadEvent::TimerReset)
                                    .unwrap_or(());
                            }
                        }
                    }
                    if let Some(hot_key) = config.hot_key_undo {
                        if input.consume_key(hot_key.modifiers, hot_key.key) {
                            // TODO: fix this unwrap
                            self.timer.write().unwrap().undo_split();
                        }
                    }
                    if let Some(hot_key) = config.hot_key_skip {
                        if input.consume_key(hot_key.modifiers, hot_key.key) {
                            // TODO: fix this unwrap
                            self.timer.write().unwrap().skip_split();
                        }
                    }
                    if let Some(hot_key) = config.hot_key_pause {
                        if input.consume_key(hot_key.modifiers, hot_key.key) {
                            // TODO: fix this unwrap
                            self.timer.write().unwrap().toggle_pause();
                        }
                    }
                    if let Some(hot_key) = config.hot_key_comparison_next {
                        if input.consume_key(hot_key.modifiers, hot_key.key) {
                            // TODO: fix this unwrap
                            self.timer.write().unwrap().switch_to_next_comparison();
                        }
                    }
                    if let Some(hot_key) = config.hot_key_comparison_prev {
                        if input.consume_key(hot_key.modifiers, hot_key.key) {
                            // TODO: fix this unwrap
                            self.timer.write().unwrap().switch_to_previous_comparison();
                        }
                    }
                });
            }
        }

        //println!("Time to update: {}μs", update_timer.elapsed().as_micros());
    }
}

pub fn app_init(
    app: &mut LiveSplitCoreRenderer,
    sync_receiver: std::sync::mpsc::Receiver<ThreadEvent>,
    cc: &eframe::CreationContext,
) {
    let context = cc.egui_ctx.clone();
    context.set_visuals(egui::Visuals::dark());
    app.load_app_config();
    if app.app_config.read().unwrap().global_hotkeys == Some(YesOrNo::Yes) {
        messagebox_on_error(|| app.enable_global_hotkeys());
    }
    let frame_rate = app
        .app_config
        .read()
        .unwrap()
        .frame_rate
        .unwrap_or(DEFAULT_FRAME_RATE);
    let polling_rate = app
        .app_config
        .read()
        .unwrap()
        .polling_rate
        .unwrap_or(DEFAULT_POLLING_RATE);
    // This thread is essentially just a refresh rate timer
    // it ensures that the gui thread is redrawn at the requested frame_rate,
    // possibly more often.
    let _frame_rate_thread = ThreadBuilder::default()
        .name("Frame Rate Thread".to_owned())
        .priority(ThreadPriority::Min)
        .spawn(move |_| loop {
            context.clone().request_repaint();
            std::thread::sleep(std::time::Duration::from_millis(
                (1000.0 / frame_rate) as u64,
            ));
        })
        // TODO: fix this unwrap
        .unwrap();

    // The timer, settings, and app_config are all behind
    // something equivalent to Arc<RwLock<_>> so it's safe
    // to clone them and pass the clone between threads.
    let timer = app.timer.clone();
    let settings = app.settings.clone();
    let app_config = app.app_config.clone();
    // This thread deals with polling the SNES at a fixed rate.
    if app_config.read().unwrap().use_autosplitter == Some(YesOrNo::Yes) {
        let _snes_polling_thread = ThreadBuilder::default()
            .name("SNES Polling Thread".to_owned())
            // We could change this thread priority, but we probably
            // should leave it at the default to make sure we get timely
            // polling of SNES state
            .spawn(move |_| loop {
                let latency = Arc::new(RwLock::new((0.0, 0.0)));
                print_on_error(|| -> std::result::Result<(), Box<dyn Error>> {
                    let mut client = crate::usb2snes::SyncClient::connect()?;
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
                            && app_config.read().unwrap().reset_timer_on_game_reset
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
                            *latency.write() = (summary.latency_average, summary.latency_stddev);
                        }
                        // If the timer gets reset, we need to make a fresh snes state
                        if let Ok(ThreadEvent::TimerReset) = sync_receiver.try_recv() {
                            snes = SNESState::new();
                            //Reset the snes
                            if app_config.read().unwrap().reset_game_on_timer_reset
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
}
