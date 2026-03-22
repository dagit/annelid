use eframe::egui;
use livesplit_core::run::editor::Editor;
use livesplit_core::run::editor::SelectionState;
use livesplit_core::run::editor::State as EditorSnapshot;
use livesplit_core::settings::ImageCache;
use livesplit_core::TimingMethod;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::livesplit_renderer::LiveSplitCoreRenderer;
use crate::ui::control_panel::UiAction;

pub(crate) struct SplitsEditorState {
    pub editor: Editor,
    pub image_cache: ImageCache,
    // Run metadata buffers
    pub game_name: String,
    pub category_name: String,
    pub offset: String,
    pub attempts: String,
    // Active segment buffers
    pub segment_name: String,
    pub split_time: String,
    pub segment_time: String,
    pub best_segment_time: String,
    pub comparison_times: Vec<String>,
    // Tracking
    pub active_index: usize,
    pub scroll_to_active: bool,
    // Comparison management
    pub new_comparison_name: String,
}

impl SplitsEditorState {
    pub fn new(editor: Editor) -> Self {
        let mut image_cache = ImageCache::new();
        let state = editor.state(&mut image_cache, livesplit_core::Lang::English);
        let active_segment = &state.segments[0];
        SplitsEditorState {
            game_name: state.game,
            category_name: state.category,
            offset: state.offset,
            attempts: state.attempts.to_string(),
            segment_name: active_segment.name.clone(),
            split_time: active_segment.split_time.clone(),
            segment_time: active_segment.segment_time.clone(),
            best_segment_time: active_segment.best_segment_time.clone(),
            comparison_times: active_segment.comparison_times.clone(),
            active_index: 0,
            scroll_to_active: true,
            new_comparison_name: String::new(),
            editor,
            image_cache,
        }
    }

    fn reload_segment_buffers(&mut self) {
        let state = self
            .editor
            .state(&mut self.image_cache, livesplit_core::Lang::English);
        if let Some(seg) = state.segments.get(self.active_index) {
            self.segment_name = seg.name.clone();
            self.split_time = seg.split_time.clone();
            self.segment_time = seg.segment_time.clone();
            self.best_segment_time = seg.best_segment_time.clone();
            self.comparison_times = seg.comparison_times.clone();
        }
    }

    fn select_segment(&mut self, index: usize) {
        if index == self.active_index {
            return;
        }
        // Flush current segment name (times are committed individually via lost_focus)
        self.editor
            .active_segment()
            .set_name(self.segment_name.as_str());
        // Select new
        self.editor.select_only(index);
        self.active_index = index;
        self.scroll_to_active = true;
        self.reload_segment_buffers();
    }

    fn update_after_mutation(&mut self) {
        let state = self
            .editor
            .state(&mut self.image_cache, livesplit_core::Lang::English);
        self.active_index = state
            .segments
            .iter()
            .position(|s| s.selected == SelectionState::Active)
            .unwrap_or(0);
        if let Some(seg) = state.segments.get(self.active_index) {
            self.segment_name = seg.name.clone();
            self.split_time = seg.split_time.clone();
            self.segment_time = seg.segment_time.clone();
            self.best_segment_time = seg.best_segment_time.clone();
            self.comparison_times = seg.comparison_times.clone();
        }
        self.scroll_to_active = true;
    }
}

#[derive(PartialEq)]
enum EditorAction {
    None,
    Update,
    SaveToFile,
    Cancel,
}

fn show_metadata_section(ui: &mut egui::Ui, es: &mut SplitsEditorState) {
    egui::Grid::new("metadata_grid")
        .num_columns(4)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("Game:");
            ui.text_edit_singleline(&mut es.game_name);
            ui.label("Category:");
            ui.text_edit_singleline(&mut es.category_name);
            ui.end_row();

            es.editor.set_game_name(es.game_name.as_str());
            es.editor.set_category_name(es.category_name.as_str());

            ui.label("Offset:");
            let offset_response = ui.text_edit_singleline(&mut es.offset);
            if offset_response.lost_focus() {
                let _ = es
                    .editor
                    .parse_and_set_offset(&es.offset, livesplit_core::Lang::English);
            }
            ui.label("Attempts:");
            let attempts_response = ui.text_edit_singleline(&mut es.attempts);
            if attempts_response.lost_focus() {
                let _ = es.editor.parse_and_set_attempt_count(&es.attempts);
            }
            ui.end_row();
        });
}

fn show_timing_method_section(ui: &mut egui::Ui, es: &mut SplitsEditorState) {
    ui.horizontal(|ui| {
        ui.label("Timing:");
        let mut current = es.editor.selected_timing_method();
        let changed_rt = ui
            .radio_value(&mut current, TimingMethod::RealTime, "Real Time")
            .changed();
        let changed_gt = ui
            .radio_value(&mut current, TimingMethod::GameTime, "Game Time")
            .changed();
        if changed_rt || changed_gt {
            es.editor.select_timing_method(current);
            es.reload_segment_buffers();
        }
    });
}

fn show_segment_table(
    ui: &mut egui::Ui,
    es: &mut SplitsEditorState,
    snapshot: &EditorSnapshot,
) -> Option<usize> {
    let mut new_selection: Option<usize> = None;

    egui::CollapsingHeader::new("Segments")
        .default_open(true)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    egui::Grid::new("segment_table")
                        .striped(true)
                        .num_columns(3)
                        .show(ui, |ui| {
                            ui.strong("#");
                            ui.strong("Name");
                            ui.strong("Split Time");
                            ui.end_row();

                            for (i, seg) in snapshot.segments.iter().enumerate() {
                                let is_active = seg.selected == SelectionState::Active;
                                let label = if is_active {
                                    format!(">{}", i + 1)
                                } else {
                                    format!(" {}", i + 1)
                                };
                                let row_response = ui.selectable_label(is_active, label);
                                if is_active && es.scroll_to_active {
                                    row_response.scroll_to_me(Some(egui::Align::Center));
                                }
                                ui.label(&seg.name);
                                ui.label(&seg.split_time);
                                ui.end_row();

                                if row_response.clicked() && !is_active {
                                    new_selection = Some(i);
                                }
                            }
                        });
                });
        });
    es.scroll_to_active = false;

    new_selection
}

fn show_segment_buttons(ui: &mut egui::Ui, es: &mut SplitsEditorState, snapshot: &EditorSnapshot) {
    ui.horizontal(|ui| {
        if ui.button("Insert Above").clicked() {
            es.editor
                .active_segment()
                .set_name(es.segment_name.as_str());
            es.editor.insert_segment_above();
            es.update_after_mutation();
        }
        if ui.button("Insert Below").clicked() {
            es.editor
                .active_segment()
                .set_name(es.segment_name.as_str());
            es.editor.insert_segment_below();
            es.update_after_mutation();
        }
        ui.add_enabled_ui(snapshot.buttons.can_remove, |ui| {
            if ui.button("Remove").clicked() {
                es.editor.remove_segments();
                es.update_after_mutation();
            }
        });
        ui.add_enabled_ui(snapshot.buttons.can_move_up, |ui| {
            if ui.button("Up").clicked() {
                es.editor.move_segments_up();
                es.update_after_mutation();
            }
        });
        ui.add_enabled_ui(snapshot.buttons.can_move_down, |ui| {
            if ui.button("Down").clicked() {
                es.editor.move_segments_down();
                es.update_after_mutation();
            }
        });
    });
}

fn show_selected_segment_detail(ui: &mut egui::Ui, es: &mut SplitsEditorState) {
    let lang = livesplit_core::Lang::English;

    egui::CollapsingHeader::new("Selected Segment")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("segment_detail")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut es.segment_name);
                    es.editor
                        .active_segment()
                        .set_name(es.segment_name.as_str());
                    ui.end_row();

                    ui.label("Split Time:");
                    let split_response = ui.text_edit_singleline(&mut es.split_time);
                    if split_response.lost_focus() {
                        let _ = es
                            .editor
                            .active_segment()
                            .parse_and_set_split_time(&es.split_time, lang);
                        es.reload_segment_buffers();
                    }
                    ui.end_row();

                    ui.label("Segment Time:");
                    let seg_response = ui.text_edit_singleline(&mut es.segment_time);
                    if seg_response.lost_focus() {
                        let _ = es
                            .editor
                            .active_segment()
                            .parse_and_set_segment_time(&es.segment_time, lang);
                        es.reload_segment_buffers();
                    }
                    ui.end_row();

                    ui.label("Best Segment:");
                    let best_response = ui.text_edit_singleline(&mut es.best_segment_time);
                    if best_response.lost_focus() {
                        let _ = es
                            .editor
                            .active_segment()
                            .parse_and_set_best_segment_time(&es.best_segment_time, lang);
                    }
                    ui.end_row();

                    // Comparison times
                    let comparison_names: Vec<String> = es.editor.custom_comparisons().to_vec();
                    for (i, comp_name) in comparison_names.iter().enumerate() {
                        while es.comparison_times.len() <= i {
                            es.comparison_times.push(String::new());
                        }
                        ui.label(format!("{}:", comp_name));
                        let comp_response = ui.text_edit_singleline(&mut es.comparison_times[i]);
                        if comp_response.lost_focus() {
                            let _ = es.editor.active_segment().parse_and_set_comparison_time(
                                comp_name,
                                &es.comparison_times[i],
                                lang,
                            );
                        }
                        ui.end_row();
                    }
                });
        });
}

fn show_comparison_management(ui: &mut egui::Ui, es: &mut SplitsEditorState) {
    egui::CollapsingHeader::new("Comparisons")
        .default_open(false)
        .show(ui, |ui| {
            let comparison_names: Vec<String> = es.editor.custom_comparisons().to_vec();

            let mut to_remove: Option<String> = None;
            for name in &comparison_names {
                ui.horizontal(|ui| {
                    ui.label(name);
                    if ui.small_button("x").clicked() {
                        to_remove = Some(name.clone());
                    }
                });
            }
            if let Some(name) = to_remove {
                es.editor.remove_comparison(&name);
                es.reload_segment_buffers();
            }

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut es.new_comparison_name);
                if ui.button("Add").clicked() && !es.new_comparison_name.is_empty() {
                    let _ = es.editor.add_comparison(es.new_comparison_name.as_str());
                    es.new_comparison_name.clear();
                    es.reload_segment_buffers();
                }
            });
        });
}

fn show_action_buttons(ui: &mut egui::Ui) -> EditorAction {
    let mut action = EditorAction::None;
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Cancel").clicked() {
                action = EditorAction::Cancel;
            }
            if ui.button("Update").clicked() {
                action = EditorAction::Update;
            }
            if ui.button("Save as...").clicked() {
                action = EditorAction::SaveToFile;
            }
        });
    });
    action
}

fn splits_editor_ui(
    ctx: &egui::Context,
    state: &Mutex<Option<SplitsEditorState>>,
    preview_slot: &Mutex<Option<livesplit_core::Run>>,
    actions: &Mutex<Vec<UiAction>>,
    open: &AtomicBool,
) {
    if ctx.input(|i| i.viewport().close_requested()) {
        open.store(false, Ordering::Relaxed);
        state.lock().take();
        return;
    }

    let mut guard = state.lock();
    let Some(ref mut es) = *guard else {
        return;
    };

    let mut action = EditorAction::None;
    let mut deferred_selection: Option<usize> = None;

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            show_metadata_section(ui, es);
            ui.separator();
            show_timing_method_section(ui, es);
            ui.separator();
            let snapshot = es
                .editor
                .state(&mut es.image_cache, livesplit_core::Lang::English);
            deferred_selection = show_segment_table(ui, es, &snapshot);
            show_segment_buttons(ui, es, &snapshot);
            ui.separator();
            show_selected_segment_detail(ui, es);
            ui.separator();
            show_comparison_management(ui, es);
            ui.separator();
            action = show_action_buttons(ui);
        });
    });

    // Process segment selection after detail panel so lost_focus handlers
    // commit any pending edits to the current segment first.
    if let Some(idx) = deferred_selection {
        es.select_segment(idx);
    }

    // Compute preview: clone the edited run so the main window can show
    // updated segment names/times while the editor is open.
    *preview_slot.lock() = Some(es.editor.run().clone());
    ctx.request_repaint_of(egui::ViewportId::ROOT);

    match action {
        EditorAction::Update => {
            if let Some(mut es) = guard.take() {
                es.editor.set_game_name(es.game_name.as_str());
                es.editor.set_category_name(es.category_name.as_str());
                let _ = es
                    .editor
                    .parse_and_set_offset(&es.offset, livesplit_core::Lang::English);
                let _ = es.editor.parse_and_set_attempt_count(&es.attempts);
                es.editor
                    .active_segment()
                    .set_name(es.segment_name.as_str());
                let run = es.editor.close();
                actions
                    .lock()
                    .push(UiAction::ApplySplitsEdit(Box::new(run)));
            }
            open.store(false, Ordering::Relaxed);
        }
        EditorAction::SaveToFile => {
            if let Some(mut es) = guard.take() {
                es.editor.set_game_name(es.game_name.as_str());
                es.editor.set_category_name(es.category_name.as_str());
                let _ = es
                    .editor
                    .parse_and_set_offset(&es.offset, livesplit_core::Lang::English);
                let _ = es.editor.parse_and_set_attempt_count(&es.attempts);
                es.editor
                    .active_segment()
                    .set_name(es.segment_name.as_str());
                let run = es.editor.close();
                let mut lock = actions.lock();
                lock.push(UiAction::ApplySplitsEdit(Box::new(run)));
                lock.push(UiAction::SaveSplitsDialog);
            }
            open.store(false, Ordering::Relaxed);
        }
        EditorAction::Cancel => {
            guard.take();
            open.store(false, Ordering::Relaxed);
        }
        EditorAction::None => {}
    }
}

impl LiveSplitCoreRenderer {
    pub(crate) fn show_splits_editor(&mut self, ctx: &egui::Context) {
        if !self.splits_editor_open.load(Ordering::Relaxed) {
            let mut guard = self.splits_editor_state.lock();
            if guard.is_some() {
                *guard = None;
            }
            return;
        }

        let state = self.splits_editor_state.clone();
        let preview_slot = self.splits_editor_preview.clone();
        let actions = self.ui_actions.clone();
        let open = self.splits_editor_open.clone();

        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of("splits_editor"),
            egui::ViewportBuilder::default()
                .with_title("Annelid Splits Editor")
                .with_inner_size([500.0, 600.0]),
            move |ctx, _class| {
                splits_editor_ui(ctx, &state, &preview_slot, &actions, &open);
            },
        );
    }
}
