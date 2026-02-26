use crate::autosplitters::supermetroid::Settings;
use crate::livesplit_renderer::LiveSplitCoreRenderer;
use crate::ui::control_panel::UiAction;
use eframe::egui;
use parking_lot::{Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

pub fn show_children(
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

/// Renders the autosplitter settings UI in a deferred viewport.
fn settings_viewport_ui(
    ctx: &egui::Context,
    settings: &RwLock<Settings>,
    actions: &Mutex<Vec<UiAction>>,
    open: &AtomicBool,
) {
    if ctx.input(|i| i.viewport().close_requested()) {
        open.store(false, Ordering::Relaxed);
        return;
    }

    let mut close = false;

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::both().show(ui, |ui| {
            let mut settings = settings.write();
            let mut roots = settings.roots();
            show_children(&mut settings, ui, ctx, &mut roots);
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    close = true;
                }
                if ui.button("Save as...").clicked() {
                    actions.lock().push(UiAction::SaveAutosplitterDialog);
                    close = true;
                }
            });
        });
    });

    if close {
        open.store(false, Ordering::Relaxed);
    }
}

impl LiveSplitCoreRenderer {
    pub(crate) fn show_autosplitter_settings_window(&mut self, ctx: &egui::Context) {
        if !self.show_settings_editor.load(Ordering::Relaxed) {
            return;
        }

        let settings = self.settings.clone();
        let actions = self.ui_actions.clone();
        let open = self.show_settings_editor.clone();

        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of("autosplitter_settings"),
            egui::ViewportBuilder::default()
                .with_title("Autosplitter Settings")
                .with_inner_size([400.0, 500.0]),
            move |ctx, _class| {
                settings_viewport_ui(ctx, &settings, &actions, &open);
            },
        );
    }
}
