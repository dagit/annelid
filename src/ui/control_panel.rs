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
    // App
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
                    if ui.button("Import Splits").clicked() {
                        actions.lock().push(UiAction::OpenSplitsDialog);
                    }
                    if ui.button("Save Splits as...").clicked() {
                        actions.lock().push(UiAction::SaveSplitsDialog);
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
                .with_inner_size([300.0, 400.0]),
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
                    self.open_layout_dialog(&document_dir, ctx).unwrap();
                }
                UiAction::OpenSplitsDialog => {
                    self.open_splits_dialog(&document_dir).unwrap();
                }
                UiAction::SaveSplitsDialog => {
                    self.save_splits_dialog(&document_dir).unwrap();
                }
                UiAction::Start => {
                    self.timer.write().unwrap().start().ok();
                }
                UiAction::Split => {
                    self.timer.write().unwrap().split().ok();
                }
                UiAction::SkipSplit => {
                    self.timer.write().unwrap().skip_split().ok();
                }
                UiAction::UndoSplit => {
                    self.timer.write().unwrap().undo_split().ok();
                }
                UiAction::Pause => {
                    self.timer.write().unwrap().pause().ok();
                }
                UiAction::Resume => {
                    self.timer.write().unwrap().resume().ok();
                }
                UiAction::Reset => {
                    self.timer.write().unwrap().reset(true).ok();
                    if self.app_config.read().unwrap().use_autosplitter == Some(YesOrNo::Yes) {
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
                    self.open_autosplitter_dialog(&document_dir).unwrap();
                }
                UiAction::SaveAutosplitterDialog => {
                    self.save_autosplitter_dialog(&document_dir).unwrap();
                }
                UiAction::Quit => {
                    ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Close);
                }
            }
        }
    }
}
