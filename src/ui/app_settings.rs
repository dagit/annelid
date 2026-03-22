use eframe::egui;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::config::app_config::*;
use crate::hotkey::HotKey;
use crate::livesplit_renderer::LiveSplitCoreRenderer;
use crate::ui::control_panel::UiAction;

pub(crate) struct SettingsState {
    pub config: AppConfig,
    pub capturing: Option<HotkeyField>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum HotkeyField {
    Start,
    Reset,
    Undo,
    Skip,
    Pause,
    ComparisonNext,
    ComparisonPrev,
}

/// Maps a YesOrNo option to a checkbox. Returns true if the value changed.
fn yes_no_checkbox(ui: &mut egui::Ui, label: &str, value: &mut Option<YesOrNo>) {
    let mut checked = *value == Some(YesOrNo::Yes);
    if ui.checkbox(&mut checked, label).changed() {
        *value = Some(if checked { YesOrNo::Yes } else { YesOrNo::No });
    }
}

/// Formats a HotKey for display using egui's built-in shortcut formatting.
fn format_hotkey(ctx: &egui::Context, hotkey: &Option<HotKey>) -> String {
    match hotkey {
        Some(hk) => {
            let shortcut = egui::KeyboardShortcut::new(hk.modifiers, hk.key);
            ctx.format_shortcut(&shortcut)
        }
        None => "(none)".to_owned(),
    }
}

/// Renders one hotkey row with capture logic.
fn hotkey_row(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    label: &str,
    hotkey: &mut Option<HotKey>,
    field: HotkeyField,
    capturing: &mut Option<HotkeyField>,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let is_capturing = *capturing == Some(field);
        if is_capturing {
            ui.label("Press a key...");
            // Check for key events
            let events = ctx.input(|i| i.events.clone());
            for event in &events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    // Escape cancels capture
                    if *key == egui::Key::Escape {
                        *capturing = None;
                        return;
                    }
                    *hotkey = Some(HotKey {
                        key: *key,
                        modifiers: *modifiers,
                    });
                    *capturing = None;
                    return;
                }
            }
            if ui.button("Cancel").clicked() {
                *capturing = None;
            }
        } else {
            ui.label(format_hotkey(ctx, hotkey));
            if ui.button("Set").clicked() {
                *capturing = Some(field);
            }
        }
    });
}

/// Renders the app settings UI in a deferred viewport.
fn settings_panel_ui(
    ctx: &egui::Context,
    state: &Mutex<Option<SettingsState>>,
    actions: &Mutex<Vec<UiAction>>,
    open: &AtomicBool,
) {
    if ctx.input(|i| i.viewport().close_requested()) {
        open.store(false, Ordering::Relaxed);
        state.lock().take();
        return;
    }

    let Some(ref mut settings_state) = *state.lock() else {
        return;
    };
    let config = &mut settings_state.config;
    let capturing = &mut settings_state.capturing;

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // --- Rendering ---
            egui::CollapsingHeader::new("Rendering")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Renderer:");
                        let current = config.renderer.unwrap_or_default();
                        egui::ComboBox::from_id_salt("renderer")
                            .selected_text(match current {
                                RendererType::Gpu => "GPU",
                                RendererType::Software => "Software",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut config.renderer,
                                    Some(RendererType::Gpu),
                                    "GPU",
                                );
                                ui.selectable_value(
                                    &mut config.renderer,
                                    Some(RendererType::Software),
                                    "Software",
                                );
                            });
                        ui.weak("(requires restart)");
                    });
                    ui.horizontal(|ui| {
                        yes_no_checkbox(ui, "Transparent Window", &mut config.transparent_window);
                        ui.weak("(requires restart)");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Frame Rate:");
                        let mut fr = config.frame_rate.unwrap_or(DEFAULT_FRAME_RATE);
                        if ui
                            .add(egui::DragValue::new(&mut fr).range(1.0..=240.0).speed(0.5))
                            .changed()
                        {
                            config.frame_rate = Some(fr);
                        }
                    });
                });

            // --- Autosplitter ---
            egui::CollapsingHeader::new("Autosplitter")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        yes_no_checkbox(ui, "Use Autosplitter", &mut config.use_autosplitter);
                        ui.weak("(requires restart)");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Polling Rate:");
                        let mut pr = config.polling_rate.unwrap_or(DEFAULT_POLLING_RATE);
                        if ui
                            .add(egui::DragValue::new(&mut pr).range(1.0..=120.0).speed(0.5))
                            .changed()
                        {
                            config.polling_rate = Some(pr);
                        }
                        ui.weak("(requires restart)");
                    });
                    yes_no_checkbox(
                        ui,
                        "Reset timer on game reset",
                        &mut config.reset_timer_on_game_reset,
                    );
                    yes_no_checkbox(
                        ui,
                        "Reset game on timer reset",
                        &mut config.reset_game_on_timer_reset,
                    );
                });

            // --- Input ---
            egui::CollapsingHeader::new("Input")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        yes_no_checkbox(ui, "Global Hotkeys", &mut config.global_hotkeys);
                        ui.weak("(requires restart)");
                    });
                });

            // --- Hotkeys ---
            egui::CollapsingHeader::new("Hotkeys")
                .default_open(true)
                .show(ui, |ui| {
                    hotkey_row(
                        ui,
                        ctx,
                        "Start:",
                        &mut config.hot_key_start,
                        HotkeyField::Start,
                        capturing,
                    );
                    hotkey_row(
                        ui,
                        ctx,
                        "Reset:",
                        &mut config.hot_key_reset,
                        HotkeyField::Reset,
                        capturing,
                    );
                    hotkey_row(
                        ui,
                        ctx,
                        "Undo:",
                        &mut config.hot_key_undo,
                        HotkeyField::Undo,
                        capturing,
                    );
                    hotkey_row(
                        ui,
                        ctx,
                        "Skip:",
                        &mut config.hot_key_skip,
                        HotkeyField::Skip,
                        capturing,
                    );
                    hotkey_row(
                        ui,
                        ctx,
                        "Pause:",
                        &mut config.hot_key_pause,
                        HotkeyField::Pause,
                        capturing,
                    );
                    hotkey_row(
                        ui,
                        ctx,
                        "Next Comparison:",
                        &mut config.hot_key_comparison_next,
                        HotkeyField::ComparisonNext,
                        capturing,
                    );
                    hotkey_row(
                        ui,
                        ctx,
                        "Prev Comparison:",
                        &mut config.hot_key_comparison_prev,
                        HotkeyField::ComparisonPrev,
                        capturing,
                    );
                });

            ui.separator();

            // --- Buttons ---
            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    actions.lock().push(UiAction::ApplySettings(config.clone()));
                }
                if ui.button("Save").clicked() {
                    actions.lock().push(UiAction::ApplySettings(config.clone()));
                    open.store(false, Ordering::Relaxed);
                }
                if ui.button("Cancel").clicked() {
                    open.store(false, Ordering::Relaxed);
                }
            });
        });
    });
}

impl LiveSplitCoreRenderer {
    pub(crate) fn show_app_settings(&mut self, ctx: &egui::Context) {
        if !self.settings_panel_open.load(Ordering::Relaxed) {
            // Clear state when panel is closed
            let mut guard = self.settings_panel_state.lock();
            if guard.is_some() {
                *guard = None;
            }
            return;
        }

        let state = self.settings_panel_state.clone();
        let actions = self.ui_actions.clone();
        let open = self.settings_panel_open.clone();

        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of("app_settings"),
            egui::ViewportBuilder::default()
                .with_title("Annelid Settings")
                .with_inner_size([450.0, 550.0]),
            move |ctx, _class| {
                settings_panel_ui(ctx, &state, &actions, &open);
            },
        );
    }
}
