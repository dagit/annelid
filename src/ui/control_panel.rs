use eframe::egui;
use livesplit_core::SharedTimer;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::autosplitters::supermetroid::Settings;
use crate::config::app_config::*;
use crate::livesplit_renderer::{LiveSplitCoreRenderer, ThreadEvent};

pub(crate) enum UiAction {
    // File
    OpenLayoutDialog,
    OpenSplitsDialog,
    SaveSplitsDialog,
    SaveLayoutDialog,
    // Timer
    Start,
    Split,
    SkipSplit,
    UndoSplit,
    Pause,
    Resume,
    Reset,
    // Autosplitter
    NewAutosplitter,
    ConfigureAutosplitter,
    OpenAutosplitterDialog,
    SaveAutosplitterDialog,
    // Splits Editor
    OpenSplitsEditor,
    ApplySplitsEdit(Box<livesplit_core::Run>),
    // Layout Editor
    OpenLayoutEditor,
    ApplyLayoutEdit(Box<livesplit_core::Layout>),
    // App
    OpenSettingsPanel,
    ApplySettings(AppConfig),
    OpenLogViewer,
    OpenLogDirectory,
    Quit,
}

/// Renders the control panel UI in the deferred viewport.
/// This is a free function (not a method on LiveSplitCoreRenderer) because
/// `show_viewport_deferred` requires `Fn + Send + Sync + 'static`.
fn control_panel_ui(
    ctx: &egui::Context,
    timer: &SharedTimer,
    actions: &Mutex<Vec<UiAction>>,
    open: &AtomicBool,
) {
    if ctx.input(|i| i.viewport().close_requested()) {
        open.store(false, Ordering::Relaxed);
        return;
    }
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::CollapsingHeader::new("File")
                .default_open(true)
                .show(ui, |ui| {
                    if ui.button("Import Layout").clicked() {
                        actions.lock().push(UiAction::OpenLayoutDialog);
                    }
                    if ui.button("Save Layout as...").clicked() {
                        actions.lock().push(UiAction::SaveLayoutDialog);
                    }
                    if ui.button("Import Splits").clicked() {
                        actions.lock().push(UiAction::OpenSplitsDialog);
                    }
                    if ui.button("Save Splits as...").clicked() {
                        actions.lock().push(UiAction::SaveSplitsDialog);
                    }
                    if ui.button("Edit Splits").clicked() {
                        actions.lock().push(UiAction::OpenSplitsEditor);
                    }
                    if ui.button("Edit Layout").clicked() {
                        actions.lock().push(UiAction::OpenLayoutEditor);
                    }
                });

            egui::CollapsingHeader::new("Run Control")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Start").clicked() {
                            actions.lock().push(UiAction::Start);
                        }
                        if ui.button("Split").clicked() {
                            actions.lock().push(UiAction::Split);
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Skip Split").clicked() {
                            actions.lock().push(UiAction::SkipSplit);
                        }
                        if ui.button("Undo Split").clicked() {
                            actions.lock().push(UiAction::UndoSplit);
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Pause").clicked() {
                            actions.lock().push(UiAction::Pause);
                        }
                        if ui.button("Resume").clicked() {
                            actions.lock().push(UiAction::Resume);
                        }
                    });
                    if ui.button("Reset").clicked() {
                        actions.lock().push(UiAction::Reset);
                    }
                });

            egui::CollapsingHeader::new("Autosplitter")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("New").clicked() {
                            actions.lock().push(UiAction::NewAutosplitter);
                        }
                        if ui.button("Configure").clicked() {
                            actions.lock().push(UiAction::ConfigureAutosplitter);
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Load Config").clicked() {
                            actions.lock().push(UiAction::OpenAutosplitterDialog);
                        }
                        if ui.button("Save Config").clicked() {
                            actions.lock().push(UiAction::SaveAutosplitterDialog);
                        }
                    });
                });

            ui.separator();
            if let Ok(guard) = timer.read() {
                ui.label(format!("Comparison: {}", guard.current_comparison()));
            }
            ui.separator();

            if ui.button("Settings").clicked() {
                actions.lock().push(UiAction::OpenSettingsPanel);
            }
            ui.horizontal(|ui| {
                if ui.button("View Log").clicked() {
                    actions.lock().push(UiAction::OpenLogViewer);
                }
                if ui.button("Open Log Directory").clicked() {
                    actions.lock().push(UiAction::OpenLogDirectory);
                }
            });
            if ui.button("Quit").clicked() {
                actions.lock().push(UiAction::Quit);
            }
        });
    });
}

impl LiveSplitCoreRenderer {
    pub(crate) fn show_control_panel(&mut self, ctx: &egui::Context) {
        if !self.control_panel_open.load(Ordering::Relaxed) {
            return;
        }

        let timer = self.timer.clone();
        let actions = self.ui_actions.clone();
        let open = self.control_panel_open.clone();

        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of("control_panel"),
            egui::ViewportBuilder::default()
                .with_title("Annelid Control Panel")
                .with_inner_size([300.0, 440.0]),
            move |ctx, _class| {
                control_panel_ui(ctx, &timer, &actions, &open);
            },
        );
    }

    pub(crate) fn process_ui_actions(&mut self, ctx: &egui::Context) {
        let empty_path = "".to_owned();
        let document_dir = match directories::UserDirs::new() {
            None => empty_path,
            Some(d) => match d.document_dir() {
                None => empty_path,
                Some(d) => d.to_str().unwrap_or("").to_owned(),
            },
        };
        let actions: Vec<UiAction> = {
            let mut guard = self.ui_actions.lock();
            std::mem::take(&mut *guard)
        };
        for action in actions {
            match action {
                UiAction::OpenLayoutDialog => {
                    if let Err(e) = self.open_layout_dialog(&document_dir, ctx) {
                        tracing::warn!("Failed to open layout dialog: {e}");
                    }
                }
                UiAction::OpenSplitsDialog => {
                    if let Err(e) = self.open_splits_dialog(&document_dir) {
                        tracing::warn!("Failed to open splits dialog: {e}");
                    }
                }
                UiAction::SaveSplitsDialog => {
                    if let Err(e) = self.save_splits_dialog(&document_dir) {
                        tracing::warn!("Failed to save splits: {e}");
                    }
                }
                UiAction::SaveLayoutDialog => {
                    if let Err(e) = self.save_layout_dialog(&document_dir, ctx) {
                        tracing::warn!("Failed to save layout: {e}");
                    }
                }
                UiAction::Start => {
                    if let Ok(mut t) = self.timer.write() {
                        t.start().ok();
                    }
                }
                UiAction::Split => {
                    if let Ok(mut t) = self.timer.write() {
                        t.split().ok();
                    }
                }
                UiAction::SkipSplit => {
                    if let Ok(mut t) = self.timer.write() {
                        t.skip_split().ok();
                    }
                }
                UiAction::UndoSplit => {
                    if let Ok(mut t) = self.timer.write() {
                        t.undo_split().ok();
                    }
                }
                UiAction::Pause => {
                    if let Ok(mut t) = self.timer.write() {
                        t.pause().ok();
                    }
                }
                UiAction::Resume => {
                    if let Ok(mut t) = self.timer.write() {
                        t.resume().ok();
                    }
                }
                UiAction::Reset => {
                    if let Ok(mut t) = self.timer.write() {
                        t.reset(true).ok();
                    }
                    if self.app_config.read().use_autosplitter == Some(YesOrNo::Yes) {
                        self.thread_chan
                            .try_send(ThreadEvent::TimerReset)
                            .unwrap_or(());
                    }
                }
                UiAction::NewAutosplitter => {
                    let mut guard = self.settings.write();
                    *guard = Settings::new();
                    drop(guard);
                    self.show_settings_editor.store(true, Ordering::Relaxed);
                }
                UiAction::ConfigureAutosplitter => {
                    self.show_settings_editor.store(true, Ordering::Relaxed);
                }
                UiAction::OpenAutosplitterDialog => {
                    if let Err(e) = self.open_autosplitter_dialog(&document_dir) {
                        tracing::warn!("Failed to open autosplitter dialog: {e}");
                    }
                }
                UiAction::SaveAutosplitterDialog => {
                    if let Err(e) = self.save_autosplitter_dialog(&document_dir) {
                        tracing::warn!("Failed to save autosplitter: {e}");
                    }
                }
                UiAction::OpenSplitsEditor => {
                    if !self
                        .splits_editor_open
                        .load(std::sync::atomic::Ordering::Relaxed)
                    {
                        let run = match self.timer.read() {
                            Ok(guard) => guard.run().clone(),
                            Err(e) => {
                                tracing::warn!("Failed to read timer for splits editor: {e}");
                                continue;
                            }
                        };
                        match livesplit_core::run::editor::Editor::new(run) {
                            Ok(editor) => {
                                let editor_state =
                                    crate::ui::splits_editor::SplitsEditorState::new(editor);
                                *self.splits_editor_state.lock() = Some(editor_state);
                                self.splits_editor_open
                                    .store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to open splits editor: {e}");
                            }
                        }
                    }
                }
                UiAction::ApplySplitsEdit(run) => {
                    if let Ok(mut t) = self.timer.write() {
                        let _ = t.set_run(*run);
                    }
                }
                UiAction::OpenLayoutEditor => {
                    if !self
                        .layout_editor_open
                        .load(std::sync::atomic::Ordering::Relaxed)
                    {
                        let layout = self.layout.clone();
                        match livesplit_core::layout::editor::Editor::new(layout) {
                            Ok(editor) => {
                                let editor_state =
                                    crate::ui::layout_editor::LayoutEditorState::new(editor);
                                *self.layout_editor_state.lock() = Some(editor_state);
                                self.layout_editor_open
                                    .store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to open layout editor: {e}");
                            }
                        }
                    }
                }
                UiAction::ApplyLayoutEdit(layout) => {
                    self.layout = *layout;
                    self.layout_modified = true;
                    // Force layout_state to re-create from the new layout
                    *self.layout_state.write() = None;
                }
                UiAction::OpenSettingsPanel => {
                    self.settings_panel_open.store(true, Ordering::Relaxed);
                    let config = self.app_config.read().clone();
                    *self.settings_panel_state.lock() =
                        Some(crate::ui::app_settings::SettingsState {
                            config,
                            capturing: None,
                        });
                }
                UiAction::ApplySettings(new_config) => {
                    *self.app_config.write() = new_config;
                    self.save_app_config();
                }
                UiAction::OpenLogViewer => {
                    self.log_viewer_open
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                }
                UiAction::OpenLogDirectory => {
                    crate::logging::open_log_dir();
                }
                UiAction::Quit => {
                    ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Close);
                }
            }
        }
    }
}
