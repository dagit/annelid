use anyhow::{anyhow, Result};
use eframe::egui;

use crate::config::app_config::*;
use crate::config::layout_meta::LayoutMeta;
use crate::livesplit_renderer::LiveSplitCoreRenderer;
use crate::utils::*;

/// Inject annelid window metadata into a layout JSON value.
/// The metadata is stored under the `"annelid"` key at the top level.
pub fn inject_layout_meta(
    layout_json: &mut serde_json::Value,
    meta: &LayoutMeta,
) -> Result<(), serde_json::Error> {
    if let serde_json::Value::Object(ref mut map) = layout_json {
        map.insert("annelid".to_owned(), serde_json::to_value(meta)?);
    }
    Ok(())
}

impl LiveSplitCoreRenderer {
    // TODO: we need to update this so that whatever the file is saved as becomes the default file
    // to load next time.
    pub fn confirm_save(
        &mut self,
        gl: &std::sync::Arc<eframe::glow::Context>,
        ctx: &egui::Context,
    ) -> Result<()> {
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
        // Check if layout needs saving: either the editor changed it, or the
        // user moved/resized the window beyond the WM's normal adjustments.
        let layout_changed = self.layout_modified || {
            if let Some(ref saved) = self.saved_layout_meta {
                let current = LayoutMeta::from_context(ctx);
                saved.differs_from(&current)
            } else {
                false
            }
        };
        if layout_changed {
            let save_requested = MessageDialog::new()
                .set_level(MessageLevel::Warning)
                .set_title("Save Layout")
                .set_description("Layout has been modified. Save layout?")
                .set_buttons(MessageButtons::YesNo)
                .show();
            if save_requested == MessageDialogResult::Yes {
                self.save_layout_dialog(&document_dir, ctx)?;
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

    pub fn save_app_config(&self) {
        messagebox_on_error(|| {
            use std::io::Write;
            let mut config_path = self.project_dirs.preference_dir().to_path_buf();
            config_path.push("settings.toml");
            tracing::debug!("Saving to {config_path:#?}");
            let f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(config_path)?;
            let mut writer = std::io::BufWriter::new(f);
            let toml = toml::to_string_pretty(&*self.app_config.read())?;
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
            tracing::debug!("Loading from {config_path:#?}");
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
            let cli_config = self.app_config.read().clone();
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
            if cli_config.renderer.is_some() {
                new_app_config.renderer = cli_config.renderer;
            }
            if cli_config.transparent_window.is_some() {
                new_app_config.transparent_window = cli_config.transparent_window;
            }
            // Hack to allow GPU rendering by default
            let defaults = AppConfig::default();
            if new_app_config.renderer.is_none() {
                new_app_config.renderer = defaults.renderer;
            }
            *self.app_config.write() = new_app_config;
            Ok(())
        });
    }

    pub fn process_app_config(&mut self, ctx: &egui::Context) {
        use anyhow::Context;
        let mut queue = vec![];
        std::mem::swap(&mut queue, &mut self.load_errors);
        queue_on_error(&mut queue, || {
            // Now that we've converged on a config, try loading what we can
            let config = self.app_config.read().clone();
            if let Some(layout) = config.recent_layout {
                self.load_layout(&std::path::PathBuf::from(&layout), ctx)
                    .with_context(|| format!("Failed to load layout file \"{layout}\""))?;
            }
            if let Some(splits) = config.recent_splits {
                let path = std::path::Path::new(&splits);
                self.load_splits(path.to_path_buf())
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

    pub fn load_layout(&mut self, path: &std::path::PathBuf, ctx: &egui::Context) -> Result<()> {
        let layout1 = self.load_original_livesplit_layout(ctx, path);
        let layout2 = self.load_livesplit_one_layout(ctx, path);
        match (layout1, layout2) {
            (Err(e1), Err(e2)) => Err(anyhow!("Failed to load file as either LiveSplit or LiveSplit One.\nErrors: LiveSplit: {e1}\nLiveSplit One: {e2}")),
            (_, _) => {
                self.layout_modified = false;
                // Store the loaded geometry as the reference for change detection
                self.saved_layout_meta =
                    crate::config::layout_meta::LayoutMeta::from_layout_file(path);
                Ok(())
            }
        }
    }

    fn load_original_livesplit_layout(
        &mut self,
        ctx: &egui::Context,
        path: &std::path::PathBuf,
    ) -> Result<()> {
        use std::io::Read;
        let f = std::fs::File::open(path)?;
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
                    Some(())
                });
                let meta = LayoutMeta {
                    window_x: x,
                    window_y: y,
                    window_width: width,
                    window_height: height,
                };
                meta.apply_to_context(ctx);
            }
        });
        Ok(())
    }

    fn load_livesplit_one_layout(
        &mut self,
        ctx: &egui::Context,
        path: &std::path::Path,
    ) -> Result<()> {
        use std::io::Read;
        let mut contents = String::new();
        std::fs::File::open(path)?.read_to_string(&mut contents)?;

        // Extract annelid window metadata if present
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
            if let Some(annelid_val) = json.get("annelid") {
                if let Ok(meta) = serde_json::from_value::<LayoutMeta>(annelid_val.clone()) {
                    meta.apply_to_context(ctx);
                }
            }
        }

        // Parse the layout settings (ignores the "annelid" key)
        self.layout = livesplit_core::layout::Layout::from_settings(
            livesplit_core::layout::LayoutSettings::from_json(contents.as_bytes())?,
        );
        Ok(())
    }

    pub fn load_splits(&mut self, path: std::path::PathBuf) -> Result<()> {
        use livesplit_core::run::parser::composite;
        use livesplit_core::Timer;
        use std::io::{BufReader, Read};
        let f = std::fs::File::open(&path)?;
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
            .recent_splits
            .as_ref()
            .and_then(|p| {
                std::path::Path::new(p)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_owned())
            })
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
            .recent_autosplitter
            .as_ref()
            .and_then(|p| {
                std::path::Path::new(p)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_owned())
            })
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

    pub fn save_layout_dialog(&mut self, default_dir: &str, ctx: &egui::Context) -> Result<()> {
        let layout_path: String = self
            .app_config
            .read()
            .recent_layout
            .clone()
            .unwrap_or_else(|| "annelid.ls1l".to_owned());
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .recent_layout
            .as_ref()
            .map_or(default_path_buf.clone(), |p| {
                let path = std::path::Path::new(&p);
                path.parent().map_or(default_path_buf, |p| p.to_path_buf())
            })
            .into_os_string()
            .into_string()
            .expect("utf8");
        // Build default filename: use basename of recent path, or fallback
        let default_fname = std::path::Path::new(&layout_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("annelid.ls1l")
            .to_owned();

        let meta = LayoutMeta::from_context(ctx);
        use rfd::FileDialog;
        messagebox_on_error(|| {
            let path = FileDialog::new()
                .set_directory(&dir)
                .set_file_name(&default_fname)
                .add_filter("LiveSplit One Layout", &["ls1l"])
                .add_filter("Any file", &["*"])
                .save_file();
            let path = match path {
                Some(path) => path,
                None => return Ok(()),
            };
            // Serialize layout settings to JSON, then inject annelid metadata
            let settings = self.layout.settings();
            let mut json = serde_json::to_value(&settings)?;
            inject_layout_meta(&mut json, &meta)?;
            let f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&path)?;
            serde_json::to_writer_pretty(f, &json)?;
            // Update recent layout path
            self.app_config.write().recent_layout =
                Some(path.into_os_string().into_string().expect("utf8"));
            self.layout_modified = false;
            // Update the reference geometry to what we just saved
            self.saved_layout_meta = Some(meta.clone());
            Ok(())
        });
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
            .recent_layout
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
            &[
                ("LiveSplit Layout", "lsl"),
                ("Livesplit One layout", "ls1l"),
            ],
            |me, path| {
                me.load_layout(&path, ctx)?;
                me.app_config.write().recent_layout =
                    Some(path.into_os_string().into_string().expect("utf8"));
                Ok(())
            },
        );
        Ok(())
    }

    pub fn open_splits_dialog(&mut self, default_dir: &str) -> Result<()> {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .recent_splits
            .as_ref()
            .map_or(default_path_buf.clone(), |p| {
                let path = std::path::Path::new(&p);
                path.parent().map_or(default_path_buf, |p| p.to_path_buf())
            })
            .into_os_string()
            .into_string()
            .expect("utf8");
        self.open_dialog(&dir, &[("LiveSplit Splits", "lss")], |me, path| {
            me.load_splits(path.clone())?;
            me.app_config.write().recent_splits =
                Some(path.into_os_string().into_string().expect("utf8"));
            Ok(())
        });
        Ok(())
    }

    pub fn open_autosplitter_dialog(&mut self, default_dir: &str) -> Result<()> {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
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
            &[("Autosplitter Configuration", "asc")],
            |me, path| {
                let f = std::fs::File::open(path.clone())?;
                me.load_autosplitter(&f)?;
                me.app_config.write().recent_autosplitter =
                    Some(path.into_os_string().into_string().expect("utf8"));
                Ok(())
            },
        );
        Ok(())
    }

    pub fn open_dialog(
        &mut self,
        default_dir: &str,
        file_types: &[(&str, &str)],
        open_action: impl FnOnce(&mut Self, std::path::PathBuf) -> Result<()>,
    ) {
        use rfd::FileDialog;
        messagebox_on_error(|| {
            let mut dialog = FileDialog::new().set_directory(default_dir);
            for file_type in file_types {
                dialog = dialog.add_filter(file_type.0, &[file_type.1])
            }
            let path = dialog.add_filter("Any file", &["*"]).pick_file();
            let path = match path {
                Some(path) => path,
                None => return Ok(()),
            };
            open_action(self, path)?;
            Ok(())
        });
    }
}
