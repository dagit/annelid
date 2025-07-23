use crate::autosplitters;
use crate::autosplitters::nwa;
use crate::autosplitters::{supermetroid::{Settings, SuperMetroidAutoSplitter}, AutoSplitter};
use crate::autosplitters::Game;
use anyhow::{anyhow, Context, Result};
use eframe::egui;
use egui::{Key, Modifiers};
use livesplit_core::{Layout, SharedTimer, Timer};
use livesplit_hotkey::Hook;
use parking_lot::RwLock;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::net::Ipv4Addr;
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
    image_cache: livesplit_core::settings::ImageCache,
    timer: SharedTimer,
    settings: Arc<RwLock<autosplitters::supermetroid::Settings>>,
    can_exit: bool,
    is_exiting: bool,
    thread_chan: std::sync::mpsc::SyncSender<ThreadEvent>,
    pub app_config: Arc<std::sync::RwLock<AppConfig>>,
    app_config_processed: bool,
    glow_canvas: GlowCanvas,
    global_hotkey_hook: Option<Hook>,
    load_errors: Vec<anyhow::Error>,
    show_edit_autosplitter_settings_dialog: Arc<AtomicBool>,
    game: Game,
    // address: Ipv4Addr,
    // port: u32,
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
                            if !settings.lookup(key) {
                                ui.disable();
                            }
                            show_children(settings, ui, ctx, &mut children);
                        });
                    });
                });
        } else {
            ui.scope(|ui| {
                ui.checkbox(settings.lookup_mut(key), key);
            });
        }
    });
}

impl LiveSplitCoreRenderer {
    pub fn new(
        timer: SharedTimer,
        layout: Layout,
        // settings: Arc<RwLock<Settings>>,
        chan: std::sync::mpsc::SyncSender<ThreadEvent>,
        config: AppConfig,
    ) -> Self {
        LiveSplitCoreRenderer {
            timer,
            layout,
            renderer: livesplit_core::rendering::software::BorrowedRenderer::new(),
            image_cache: livesplit_core::settings::ImageCache::new(),
            layout_state: None,
            // show_settings_editor: false,
            settings: Arc::new(
                RwLock::new(autosplitters::supermetroid::Settings::new()),
            ),
            can_exit: false,
            is_exiting: false,
            thread_chan: chan,
            app_config: Arc::new(std::sync::RwLock::new(config)),
            app_config_processed: false,
            glow_canvas: GlowCanvas::new(),
            global_hotkey_hook: None,
            load_errors: vec![],
            show_edit_autosplitter_settings_dialog: Arc::new(AtomicBool::new(false)),
            game: Game::Battletoads,
            // address: Ipv4Addr::new(0, 0, 0, 0),
            // port: 48879,
        }
    }

    pub fn confirm_save(&mut self, gl: &Arc<eframe::glow::Context>) -> Result<()> {
        use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
        let empty_path = "".to_owned();
        let document_dir = match directories::UserDirs::new() {
            None => empty_path,
            Some(d) => match d.document_dir() {
                None => empty_path,
                Some(d) => d.to_str().unwrap_or("").to_owned(),
            },
        };
        if self
            .timer
            .read()
            .map_err(|e| anyhow!("failed to acquire write lock on timer: {e}"))?
            .run()
            .has_been_modified()
        {
            let save_requested = MessageDialog::new()
                .set_level(MessageLevel::Warning)
                .set_title("Save Splits")
                .set_description("Splits have been modified. Save splits?")
                .set_buttons(MessageButtons::YesNo)
                .show();
            if save_requested == MessageDialogResult::Yes {
                self.save_splits_dialog(&document_dir)?;
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
                self.save_autosplitter_dialog(&document_dir)?;
            }
        }
        self.can_exit = true;
        self.glow_canvas.destroy(gl);
        Ok(())
    }

    pub fn process_app_config(&mut self, ctx: &egui::Context) {
        use anyhow::Context;
        let mut queue = vec![];
        std::mem::swap(&mut queue, &mut self.load_errors);
        queue_on_error(&mut queue, || {
            // Now that we've converged on a config, try loading what we can
            let config = self
                .app_config
                .read()
                .map_err(|e| anyhow!("failed to acquire read lock on config: {e}"))?
                .clone();
            if let Some(layout) = config.recent_layout {
                let f = std::fs::File::open(&layout)
                    .with_context(|| format!("Failed to open layout file \"{layout}\""))?;
                self.load_layout(&f, ctx)
                    .with_context(|| format!("Failed to load layout file \"{layout}\""))?;
            }
            if let Some(splits) = config.recent_splits {
                let f = std::fs::File::open(&splits)
                    .with_context(|| format!("Failed to open splits file \"{splits}\""))?;
                let path = std::path::Path::new(&splits)
                    .parent()
                    .ok_or(anyhow!("failed to find parent directory"))?;
                self.load_splits(&f, path.to_path_buf())
                    .with_context(|| format!("Failed to load splits file \"{splits}\""))?;
            }
            if let Some(autosplitter) = config.recent_autosplitter {
                let f = std::fs::File::open(&autosplitter).with_context(|| {
                    format!("Failed to open autosplitter config \"{autosplitter}\"")
                })?;
                self.load_autosplitter(&f).with_context(|| {
                    format!("Failed to load autosplitter config \"{autosplitter}\"")
                })?;
            }
            Ok(())
        });
        self.load_errors = queue;
    }

    pub fn load_layout(&mut self, f: &std::fs::File, ctx: &egui::Context) -> Result<()> {
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
                d.children().try_for_each(|d| {
                    if d.tag_name().name() == "Mode" {
                        mode = d.text();
                    }
                    if d.tag_name().name() == "X" {
                        x = d.text().and_then(|d| f32::from_str(d).ok());
                    }
                    if d.tag_name().name() == "Y" {
                        y = d.text().and_then(|d| f32::from_str(d).ok());
                    }
                    if mode.is_some() && d.tag_name().name() == format!("{}Width", mode?) {
                        width = d.text().and_then(|d| f32::from_str(d).ok());
                    }
                    if mode.is_some() && d.tag_name().name() == format!("{}Height", mode?) {
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
                    Some(())
                });
            }
        });
        Ok(())
    }

    pub fn load_splits(&mut self, f: &std::fs::File, path: std::path::PathBuf) -> Result<()> {
        use livesplit_core::run::parser::composite;
        use std::io::{BufReader, Read};
        let file_contents: std::result::Result<Vec<_>, _> = BufReader::new(f).bytes().collect();
        *self
            .timer
            .write()
            .map_err(|e| anyhow!("failed to acquire write lock on timer: {e}"))? =
            Timer::new(composite::parse(&file_contents?, path.parent())?.run)?;
        Ok(())
    }

    pub fn load_autosplitter(&mut self, f: &std::fs::File) -> Result<()> {
        *self.settings.write() = serde_json::from_reader(std::io::BufReader::new(f))?;
        Ok(())
    }

    pub fn save_splits_dialog(&mut self, default_dir: &str) -> Result<()> {
        let mut fname = self
            .timer
            .read()
            .map_err(|e| anyhow!("failed to acquire read lock on timer: {e}"))?
            .run()
            .extended_file_name(false);
        let splits = self
            .app_config
            .read()
            .map_err(|e| anyhow!("failed to acquire read lock on config: {e}"))?
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
            .map_err(|e| anyhow!("failed to acquire read lock on config: {e}"))?
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
                livesplit_core::run::saver::livesplit::save_timer(
                    &*me.timer
                        .read()
                        .map_err(|e| anyhow!("failed to acquire read lock on config: {e}"))?,
                    writer,
                )?;
                Ok(())
            },
        );
        Ok(())
    }

    pub fn save_autosplitter_dialog(&mut self, default_dir: &str) -> Result<()> {
        let mut fname = self
            .timer
            .read()
            .map_err(|e| anyhow!("failed to acquire read lock on timer: {e}"))?
            .run()
            .extended_file_name(false);
        let autosplitter: String = self
            .app_config
            .read()
            .map_err(|e| anyhow!("failed to acquire read lock on config: {e}"))?
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
            .map_err(|e| anyhow!("failed to acquire read lock on config: {e}"))?
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
        Ok(())
    }

    pub fn save_dialog(
        &mut self,
        default_dir: &str,
        default_fname: &str,
        file_type: (&str, &str),
        save_action: impl FnOnce(&mut Self, std::fs::File) -> Result<()>,
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

    pub fn open_layout_dialog(&mut self, default_dir: &str, ctx: &egui::Context) -> Result<()> {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .map_err(|e| anyhow!("failed to acquire read lock on config: {e}"))?
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
            me.app_config
                .write()
                .map_err(|e| anyhow!("failed to acquire write lock on config: {e}"))?
                .recent_layout = Some(path.into_os_string().into_string().expect("utf8"));
            Ok(())
        });
        Ok(())
    }

    pub fn open_splits_dialog(&mut self, default_dir: &str) -> Result<()> {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .map_err(|e| anyhow!("failed to acquire read lock on config: {e}"))?
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
            me.app_config
                .write()
                .map_err(|e| anyhow!("failed to acquire write lock on config: {e}"))?
                .recent_splits = Some(path.into_os_string().into_string().expect("utf8"));
            Ok(())
        });
        Ok(())
    }

    pub fn open_autosplitter_dialog(&mut self, default_dir: &str) -> Result<()> {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .map_err(|e| anyhow!("failed to acquire read lock on config: {e}"))?
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
                me.app_config
                    .write()
                    .map_err(|e| anyhow!("failed to acquire write lock on config: {e}"))?
                    .recent_autosplitter = Some(path.into_os_string().into_string().expect("utf8"));
                Ok(())
            },
        );
        Ok(())
    }

    pub fn open_dialog(
        &mut self,
        default_dir: &str,
        file_type: (&str, &str),
        open_action: impl FnOnce(&mut Self, std::fs::File, std::path::PathBuf) -> Result<()>,
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

    pub fn enable_global_hotkeys(&mut self) -> Result<()> {
        // It would be more elegant to use get_or_insert_with, however
        // the `with` branch cannot have a `Result` type if we do that.
        let hook: &Hook = match self.global_hotkey_hook.as_ref() {
            None => {
                self.global_hotkey_hook = Some(Hook::new()?);
                self.global_hotkey_hook.as_ref().unwrap() // We just set it so this will always
                                                          // succeed.
            }
            Some(h) => h,
        };

        // This is a bit of a mess but it lets us reduce a lot of duplication.
        // the idea here is that make_cb gives us a fresh callback each time
        // we clone it. That way we can register the call back twice,
        // once for the primary key and once for the alternate key.
        fn reg<F>(hook: &Hook, hot_key: &HotKey, make_cb: F) -> Result<()>
        where
            F: Fn() + Send + 'static + Clone,
        {
            // main binding
            hook.register(hot_key.to_livesplit_hotkey(), make_cb.clone())?;
            // optional “alt” binding
            if let Some(alt_code) = to_livesplit_keycode_alternative(&hot_key.key) {
                let alt = livesplit_hotkey::Hotkey {
                    key_code: alt_code,
                    modifiers: to_livesplit_modifiers(&hot_key.modifiers),
                };
                hook.register(alt, make_cb)?;
            }
            Ok(())
        }

        let cfg = self
            .app_config
            .read()
            .map_err(|e| anyhow!("failed to read config: {e}"))?;
        let timer = self.timer.clone();
        let thread_chan = self.thread_chan.clone();
        let app_cfg = self.app_config.clone();

        print!("Registering global hotkeys...");
        if let Some(hk) = cfg.hot_key_start {
            reg(hook, &hk, {
                let timer = timer.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.split_or_start().ok())
                        .map_err(|e| println!("split/start lock failed: {e}"));
                }
            })?;
        }
        if let Some(hk) = cfg.hot_key_reset {
            reg(hook, &hk, {
                let timer = timer.clone();
                let tc = thread_chan.clone();
                let app_cfg = app_cfg.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.reset(true).ok())
                        .map_err(|e| println!("reset lock failed: {e}"));
                    if app_cfg
                        .read()
                        .map(|g| g.use_autosplitter == Some(YesOrNo::Yes))
                        .unwrap_or(false)
                    {
                        tc.try_send(ThreadEvent::TimerReset).unwrap_or(());
                    }
                }
            })?;
        }
        if let Some(hk) = cfg.hot_key_undo {
            reg(hook, &hk, {
                let timer = timer.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.undo_split().ok())
                        .map_err(|e| println!("undo lock failed: {e}"));
                }
            })?;
        }
        if let Some(hk) = cfg.hot_key_skip {
            reg(hook, &hk, {
                let timer = timer.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.skip_split().ok())
                        .map_err(|e| println!("skip split lock failed: {e}"));
                }
            })?;
        }
        if let Some(hk) = cfg.hot_key_pause {
            reg(hook, &hk, {
                let timer = timer.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.toggle_pause().ok())
                        .map_err(|e| println!("toggle pause lock failed: {e}"));
                }
            })?;
        }
        if let Some(hk) = cfg.hot_key_comparison_next {
            reg(hook, &hk, {
                let timer = timer.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.switch_to_next_comparison())
                        .map_err(|e| println!("next comparison lock failed: {e}"));
                }
            })?;
        }
        if let Some(hk) = cfg.hot_key_comparison_prev {
            reg(hook, &hk, {
                let timer = timer.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.switch_to_previous_comparison())
                        .map_err(|e| println!("prev comparison lock failed: {e}"));
                }
            })?;
        }

        println!("registered");
        Ok(())
    }
    pub fn auto_splitter_settings_editor(&mut self, ctx: &egui::Context) {
        if self
            .show_edit_autosplitter_settings_dialog
            .load(Ordering::Relaxed)
        {
            let show_deferred_viewport = self.show_edit_autosplitter_settings_dialog.clone();
            let a_settings = self.settings.clone();

            ctx.show_viewport_deferred(
                egui::ViewportId::from_hash_of("deferred_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("AutoSplitter Settings Editor")
                    .with_inner_size([200.0, 500.0]),
                move |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Deferred,
                        "This egui backend doesn't support multiple viewports"
                    );
                    egui::CentralPanel::default().show(ctx, |_ui| {
                        let settings_editor = egui::containers::Window::new("Settings Editor");
                        settings_editor
                            .open(&mut show_deferred_viewport.load(Ordering::Relaxed))
                            .resizable(true)
                            .collapsible(false)
                            .hscroll(true)
                            .vscroll(true)
                            .show(ctx, |ui| {
                                ctx.move_to_top(ui.layer_id());
                                let settings = a_settings.clone();
                                let mut roots = settings.write().roots();
                                show_children(&mut settings.write(), ui, ctx, &mut roots);
                            });
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        // Tell parent to close us.
                        show_deferred_viewport.store(false, Ordering::Relaxed);
                    }
                },
            );
        }
    }
}

impl eframe::App for LiveSplitCoreRenderer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        //let update_timer = std::time::Instant::now();
        if self.app_config_processed && !self.load_errors.is_empty() {
            let mut queue: Vec<anyhow::Error> = vec![];
            std::mem::swap(&mut queue, &mut self.load_errors);
            for e in queue.into_iter() {
                messagebox_on_error(move || Err(e))
            }
        }
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
                self.confirm_save(frame.gl().expect("No GL context"))
                    .unwrap();
                self.app_config.read().unwrap().save_app_config(); // aquire read lock then save app config
            }
        });
        if self.can_exit {
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Close);
            return;
        } else {
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::CancelClose)
        }
        let viewport = ctx.input(|i| i.screen_rect);
        self.glow_canvas.update_frame_buffer(
            viewport,
            frame.gl().unwrap(),
            |frame_buffer, sz, stride| {
                {
                    let timer = self.timer.read().unwrap();
                    let snapshot = timer.snapshot();
                    match &mut self.layout_state {
                        None => {
                            self.layout_state =
                                Some(self.layout.state(&mut self.image_cache, &snapshot));
                        }
                        Some(layout_state) => {
                            self.layout.update_state(
                                layout_state,
                                &mut self.image_cache,
                                &snapshot,
                            );
                        }
                    };
                }

                if let Some(layout_state) = &self.layout_state {
                    self.renderer.render(
                        layout_state,
                        &self.image_cache,
                        frame_buffer,
                        sz,
                        stride,
                        true,
                    );
                }
            },
        );
        self.glow_canvas
            .paint_layer(ctx, egui::LayerId::background(), viewport);
        // //self.glow_canvas.paint_immediate(frame.gl().unwrap(), viewport);
        // let settings_editor = egui::containers::Window::new("Settings Editor");
        egui::Area::new("livesplit".into())
            // .enabled(!self.show_settings_editor)
            .movable(false)
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
                        self.open_layout_dialog(&document_dir, ctx).unwrap();
                    }
                    if ui.button("Import Splits").clicked() {
                        ui.close_menu();
                        self.open_splits_dialog(&document_dir).unwrap();
                    }
                    if ui.button("Save Splits as...").clicked() {
                        ui.close_menu();
                        self.save_splits_dialog(&document_dir).unwrap();
                    }
                });
                ui.menu_button("Run Control", |ui| {
                    if ui.button("Start").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().start().ok();
                        ui.close_menu()
                    }
                    if ui.button("Split").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().split().ok();
                        ui.close_menu()
                    }
                    ui.separator();
                    if ui.button("Skip Split").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().skip_split().ok();
                        ui.close_menu()
                    }
                    if ui.button("Undo Split").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().undo_split().ok();
                        ui.close_menu()
                    }
                    ui.separator();
                    if ui.button("Pause").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().pause().ok();
                        ui.close_menu()
                    }

                    if ui.button("Resume").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().resume().ok();
                        ui.close_menu()
                    }
                    ui.separator();
                    if ui.button("Reset").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().reset(true).ok();
                        if self.app_config.read().unwrap().use_autosplitter == Some(YesOrNo::Yes) {
                            self.thread_chan
                                .try_send(ThreadEvent::TimerReset)
                                .unwrap_or(());
                        }
                        ui.close_menu()
                    }
                });
                ui.menu_button("Autosplitter", |ui| {
                    ui.menu_button("NWA", |ui| {
                        if ui.button("Configure").clicked() {
                            // Fill out NWA config
                            // address
                            // port
                        }
                        // TODO: Fix this. It's not updating the value
                        egui::ComboBox::from_label("Game")
                            .selected_text(format!("{:?}", self.game))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.game,
                                    Game::Battletoads,
                                    "Battletoads",
                                );
                                ui.selectable_value(
                                    &mut self.game,
                                    Game::SuperMetroid,
                                    "Super Metroid",
                                );
                            })
                        // ui.menu_button("Battletoads", |ui| {
                        // });
                        // ui.menu_button("Super Metroid", |ui| {
                        // });
                    });
                    ui.menu_button("QUSB2SNES", |ui| {
                        ui.menu_button("Super Metroid", |ui| {
                            if ui.button("Configure").clicked() {
                                // self.show_settings_editor = true;
                                let show_deferred_viewport = true;
                                self.show_edit_autosplitter_settings_dialog
                                    .store(show_deferred_viewport, Ordering::Relaxed);
                                ui.close_menu();
                            }
                            if ui.button("Load Configuration").clicked() {
                                ui.close_menu();
                                self.open_autosplitter_dialog(&document_dir).unwrap();
                            }
                            if ui.button("Save Configuration").clicked() {
                                ui.close_menu();
                                self.save_autosplitter_dialog(&document_dir).unwrap();
                            }
                        });
                    });
                });
                ui.separator();
                ui.add(egui::widgets::Label::new(format!(
                    "Comparison: {}",
                    self.timer.read().unwrap().current_comparison()
                )));
                ui.separator();
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Close)
                }
            });

        self.auto_splitter_settings_editor(ctx);

        ctx.input(|i| {
            let scroll_delta = i.raw_scroll_delta;
            if scroll_delta.y > 0.0 {
                self.layout.scroll_up();
            } else if scroll_delta.y < 0.0 {
                self.layout.scroll_down();
            }
        });
        {
            let config = self.app_config.read().unwrap();
            if config.global_hotkeys != Some(YesOrNo::Yes) {
                ctx.input_mut(|input| {
                    if let Some(hot_key) = config.hot_key_start {
                        if input.consume_key(hot_key.modifiers, hot_key.key) {
                            // TODO: fix this unwrap
                            self.timer.write().unwrap().split_or_start().ok();
                        }
                    }
                    if let Some(hot_key) = config.hot_key_reset {
                        if input.consume_key(hot_key.modifiers, hot_key.key) {
                            // TODO: fix this unwrap
                            self.timer.write().unwrap().reset(true).ok();
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
                            self.timer.write().unwrap().undo_split().ok();
                        }
                    }
                    if let Some(hot_key) = config.hot_key_skip {
                        if input.consume_key(hot_key.modifiers, hot_key.key) {
                            // TODO: fix this unwrap
                            self.timer.write().unwrap().skip_split().ok();
                        }
                    }
                    if let Some(hot_key) = config.hot_key_pause {
                        if input.consume_key(hot_key.modifiers, hot_key.key) {
                            // TODO: fix this unwrap
                            self.timer.write().unwrap().toggle_pause().ok();
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
    // app.load_app_config();
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
    let app_config = app.app_config.clone();

    // This thread deals with polling the SNES at a fixed rate.
    if app_config.read().unwrap().use_autosplitter == Some(YesOrNo::Yes) {
        if app_config.read().unwrap().autosplitter_type == Some(autosplitters::AType::QUSB2SNES) {
            //QUSB2SNES stuff here
            let settings = app.settings.clone();
            let _snes_polling_thread = ThreadBuilder::default()
                .name("SNES Polling Thread".to_owned())
                // We could change this thread priority, but we probably
                // should leave it at the default to make sure we get timely
                // polling of SNES state
                .spawn(move |_| {
                    loop {
                        let latency = Arc::new(RwLock::new((0.0, 0.0)));
                        print_on_error(|| -> anyhow::Result<()> {
                            let mut client = crate::usb2snes::SyncClient::connect()
                                .context("creating usb2snes connection")?;
                            client.set_name("annelid")?;
                            println!("Server version is {:?}", client.app_version()?);
                            let mut devices = client.list_device()?.to_vec();
                            if devices.len() != 1 {
                                if devices.is_empty() {
                                    Err(anyhow!("No devices present"))?;
                                } else {
                                    Err(anyhow!("You need to select a device: {:#?}", devices))?;
                                }
                            }
                            let device = devices.pop().ok_or(anyhow!("Device list was empty"))?;
                            println!("Using device: {device}");
                            client.attach(&device)?;
                            println!("Connected.");
                            println!("{:#?}", client.info()?);

                            // TODO: make this generic as well based on user input or add game selector
                            let mut autosplitter: Box<dyn AutoSplitter> =
                                Box::new(SuperMetroidAutoSplitter::new(settings.clone()));
                            loop {
                                let summary = autosplitter.update(&mut client)?;
                                if summary.start {
                                    timer
                                        .write()
                                        .map_err(|e| {
                                            anyhow!("failed to acquire write lock on timer: {e}")
                                        })?
                                        .start()
                                        .ok();
                                }
                                if summary.reset
                                    && app_config
                                        .read()
                                        .map_err(|e| {
                                            anyhow!("failed to acquire read lock on config: {e}")
                                        })?
                                        .reset_timer_on_game_reset
                                        == Some(YesOrNo::Yes)
                                {
                                    timer
                                        .write()
                                        .map_err(|e| {
                                            anyhow!("failed to acquire write lock on timer: {e}")
                                        })?
                                        .reset(true)
                                        .ok();
                                }
                                if summary.split {
                                    if let Some(t) = autosplitter.gametime_to_seconds() {
                                        timer
                                            .write()
                                            .map_err(|e| {
                                                anyhow!(
                                                    "failed to acquire write lock on timer: {e}"
                                                )
                                            })?
                                            .set_game_time(t)
                                            .ok();
                                    }
                                    timer
                                        .write()
                                        .map_err(|e| {
                                            anyhow!("failed to acquire write lock on timer: {e}")
                                        })?
                                        .split()
                                        .ok();
                                }
                                {
                                    *latency.write() =
                                        (summary.latency_average, summary.latency_stddev);
                                }
                                // If the timer gets reset, we need to make a fresh snes state
                                if let Ok(ThreadEvent::TimerReset) = sync_receiver.try_recv() {
                                    autosplitter.reset_game_tracking();
                                    // Reset the snes
                                    if app_config
                                        .read()
                                        .map_err(|e| {
                                            anyhow!("failed to acquire read lock on config: {e}")
                                        })?
                                        .reset_game_on_timer_reset
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
                    }
                })
                //TODO: fix this unwrap
                .unwrap();
        } else if app_config.read().unwrap().autosplitter_type == Some(autosplitters::AType::NWA) {
            //NWA stuff here
            let game = app.game;
            let _nwa_polling_thread =
                ThreadBuilder::default()
                    .name("NWA Polling Thread".to_owned())
                    .spawn(move |_| loop {
                        print_on_error(|| -> anyhow::Result<()> {
                            // let client: battletoadsAutoSplitter = autosplitters::AutoSplitterSelector("Battletoads", true).unwrap();
                            // TODO: make this generic as well based on user input or add game selector
                            match game {
                                Game::Battletoads => {
                                    let mut client = nwa::battletoads::BattletoadsAutoSplitter::new(
                                        Ipv4Addr::new(0, 0, 0, 0),
                                        48879,
                                        app_config
                                            .read()
                                            .unwrap()
                                            .reset_timer_on_game_reset
                                            .unwrap(),
                                    );
                                    client.emu_info();
                                    client.emu_game_info();
                                    client.emu_status();
                                    client.client_id();
                                    client.core_info();
                                    client.core_memories();
                                    loop {
                                        println!("{game:#?}");
                                        let auto_split_status = client.update().unwrap();
                                        if auto_split_status.start {
                                            timer
                                    .write()
                                    .map_err(|e| {
                                        anyhow!("failed to acquire write lock on timer: {e}")
                                    })?
                                    .start()
                                    .ok();
                                        }
                                        if auto_split_status.reset {
                                            timer
                                    .write()
                                    .map_err(|e| {
                                        anyhow!("failed to acquire write lock on timer: {e}")
                                    })?
                                    .reset(true)
                                    .ok();
                                        }
                                        if auto_split_status.split {
                                            timer
                                    .write()
                                    .map_err(|e| {
                                        anyhow!("failed to acquire write lock on timer: {e}")
                                    })?
                                    .split()
                                    .ok();
                                        }

                                        std::thread::sleep(std::time::Duration::from_millis(
                                            (1000.0 / polling_rate) as u64,
                                        ));
                                    }
                                }
                                Game::SuperMetroid => {
                                    let mut client =
                                        nwa::supermetroid::SupermetroidAutoSplitter::new(
                                            Ipv4Addr::new(0, 0, 0, 0),
                                            48879,
                                            app_config
                                                .read()
                                                .unwrap()
                                                .reset_timer_on_game_reset
                                                .unwrap(),
                                        );
                                    client.emu_info();
                                    client.emu_game_info();
                                    client.emu_status();
                                    client.client_id();
                                    client.core_info();
                                    client.core_memories();
                                    loop {
                                        println!("{game:#?}");
                                        let auto_split_status = client.update().unwrap();
                                        if auto_split_status.start {
                                            timer
                                    .write()
                                    .map_err(|e| {
                                        anyhow!("failed to acquire write lock on timer: {e}")
                                    })?
                                    .start()
                                    .ok();
                                        }
                                        if auto_split_status.reset {
                                            timer
                                    .write()
                                    .map_err(|e| {
                                        anyhow!("failed to acquire write lock on timer: {e}")
                                    })?
                                    .reset(true)
                                    .ok();
                                        }
                                        if auto_split_status.split {
                                            timer
                                    .write()
                                    .map_err(|e| {
                                        anyhow!("failed to acquire write lock on timer: {e}")
                                    })?
                                    .split()
                                    .ok();
                                        }

                                        std::thread::sleep(std::time::Duration::from_millis(
                                            (1000.0 / polling_rate) as u64,
                                        ));
                                    }
                                }
                                _ => todo!(),
                            }
                            // if app.game == Game::Battletoads {

                            // } else if app.game == Game::SuperMetroid {

                            // }
                        });
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    })
                    //TODO: fix this unwrap
                    .unwrap();
        } else if app_config.read().unwrap().autosplitter_type == Some(autosplitters::AType::ASL) {
            //TODO: unable to configure runtime

            // let test = livesplit_auto_splitting::Runtime::new(module, timer, settings_store);
            // Livesplit autosplitter support
            // use livesplit_auto_splitting::*;
            // let test = ;
            // let test = livesplit_auto_splitting::Timer;
            // let module = livesplit_auto_splitting::Runtime::
            // livesplit_auto_splitting::Runtime::new(module, timer, settings_store)
            // let x = livesplit_auto_splitting::Runtime::new(module, timer.write().unwrap().deref(), settings_store);
        } else if app_config.read().unwrap().autosplitter_type == Some(autosplitters::AType::CUSTOM)
        {
            // TODO: process isn't consistently gotten
            // TODO: reading crashes with either bad address as root or permission denied as user
            // This is also linux only

            // use process_memory::*;
            // use sysinfo::*;
            // let mut x = 0_u64;
            // let x = sysinfo::Pid::from(17696).as_u32();
            // let s = System::new_all();
            // for (pid, process) in s.processes() {
            // println!("{} {:?}", pid, process.name());
            // }
            // let count = s.processes().clone().("retroarch");
            // let count = s.processes_by_exact_name(OsStr::new("retroarch")).count();
            // let p = s.processes_by_exact_name(OsStr::new("retroarch"));
            // if count == 2 {
            // x = p.last().unwrap().pid().as_u32();
            // }
            // println!("{x:?}");

            // let arch = process_memory::Architecture::from_native();
            // let process_handle = process_memory::ProcessHandle::try_into_process_handle(&(
            // x.try_into().unwrap(),
            // arch,
            // ))
            // .unwrap();
            // let mut member = DataMember::<i32>::new_offset(process_handle, vec![0x10]);
            // member.set_offset(vec![0x10]);

            // The memory offset can now be correctly calculated:
            // called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }
            // println!(
            // "Target memory location: {}",
            // member.clone().get_offset().unwrap()
            // );
            // The memory offset can now be used to retrieve and modify values:
            // println!("Current value: {}", unsafe { member.read().unwrap() });
        }
    }
}
