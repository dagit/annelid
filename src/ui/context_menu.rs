use eframe::egui;

use crate::autosplitters::supermetroid::Settings;
use crate::config::app_config::*;
use crate::livesplit_renderer::{LiveSplitCoreRenderer, ThreadEvent};

impl LiveSplitCoreRenderer {
    pub(crate) fn show_context_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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
                ui.close();
                self.open_layout_dialog(&document_dir, ctx).unwrap();
            }
            if ui.button("Import Splits").clicked() {
                ui.close();
                self.open_splits_dialog(&document_dir).unwrap();
            }
            if ui.button("Save Splits as...").clicked() {
                ui.close();
                self.save_splits_dialog(&document_dir).unwrap();
            }
        });
        ui.menu_button("Run Control", |ui| {
            if ui.button("Start").clicked() {
                // TODO: fix this unwrap
                self.timer.write().unwrap().start().ok();
                ui.close()
            }
            if ui.button("Split").clicked() {
                // TODO: fix this unwrap
                self.timer.write().unwrap().split().ok();
                ui.close()
            }
            ui.separator();
            if ui.button("Skip Split").clicked() {
                // TODO: fix this unwrap
                self.timer.write().unwrap().skip_split().ok();
                ui.close()
            }
            if ui.button("Undo Split").clicked() {
                // TODO: fix this unwrap
                self.timer.write().unwrap().undo_split().ok();
                ui.close()
            }
            ui.separator();
            if ui.button("Pause").clicked() {
                // TODO: fix this unwrap
                self.timer.write().unwrap().pause().ok();
                ui.close()
            }

            if ui.button("Resume").clicked() {
                // TODO: fix this unwrap
                self.timer.write().unwrap().resume().ok();
                ui.close()
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
                ui.close()
            }
        });
        ui.menu_button("Autosplitter", |ui| {
            if ui.button("New").clicked() {
                let mut guard = self.settings.write();
                *guard = Settings::new();
                self.show_settings_editor = true;
                ui.close();
            }
            if ui.button("Configure").clicked() {
                self.show_settings_editor = true;
                ui.close();
            }
            if ui.button("Load Configuration").clicked() {
                ui.close();
                self.open_autosplitter_dialog(&document_dir).unwrap();
            }
            if ui.button("Save Configuration").clicked() {
                ui.close();
                self.save_autosplitter_dialog(&document_dir).unwrap();
            }
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
    }
}
