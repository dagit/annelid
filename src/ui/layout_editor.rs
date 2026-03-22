use eframe::egui;
use livesplit_core::component::splits::{ColumnStartWith, ColumnUpdateTrigger, ColumnUpdateWith};
use livesplit_core::component::timer::DeltaGradient;
use livesplit_core::layout::editor::State as EditorSnapshot;
use livesplit_core::layout::{LayoutDirection, LayoutState};
use livesplit_core::settings::{
    Alignment, BackgroundImage, Color as LsColor, ColumnKind, FontStretch, FontStyle, FontWeight,
    Gradient, ImageCache, LayoutBackground, ListGradient, Value,
};
use livesplit_core::timing::formatter::{Accuracy, DigitsFormat};
use livesplit_core::SharedTimer;
use livesplit_core::TimingMethod;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::livesplit_renderer::LiveSplitCoreRenderer;
use crate::ui::control_panel::UiAction;

// ---------------------------------------------------------------------------
// ComponentKind — type-safe enum for the "Add Component" dropdown
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentKind {
    BlankSpace,
    CurrentComparison,
    CurrentPace,
    Delta,
    DetailedTimer,
    Graph,
    PbChance,
    PossibleTimeSave,
    PreviousSegment,
    SegmentTime,
    Separator,
    Splits,
    SumOfBest,
    Text,
    Timer,
    Title,
    TotalPlaytime,
}

impl ComponentKind {
    pub const ALL: &[Self] = &[
        Self::BlankSpace,
        Self::CurrentComparison,
        Self::CurrentPace,
        Self::Delta,
        Self::DetailedTimer,
        Self::Graph,
        Self::PbChance,
        Self::PossibleTimeSave,
        Self::PreviousSegment,
        Self::SegmentTime,
        Self::Separator,
        Self::Splits,
        Self::SumOfBest,
        Self::Text,
        Self::Timer,
        Self::Title,
        Self::TotalPlaytime,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            Self::BlankSpace => "Blank Space",
            Self::CurrentComparison => "Current Comparison",
            Self::CurrentPace => "Current Pace",
            Self::Delta => "Delta",
            Self::DetailedTimer => "Detailed Timer",
            Self::Graph => "Graph",
            Self::PbChance => "PB Chance",
            Self::PossibleTimeSave => "Possible Time Save",
            Self::PreviousSegment => "Previous Segment",
            Self::SegmentTime => "Segment Time",
            Self::Separator => "Separator",
            Self::Splits => "Splits",
            Self::SumOfBest => "Sum of Best",
            Self::Text => "Text",
            Self::Timer => "Timer",
            Self::Title => "Title",
            Self::TotalPlaytime => "Total Playtime",
        }
    }

    fn add_to(self, editor: &mut livesplit_core::layout::editor::Editor) {
        use livesplit_core::component::*;
        match self {
            Self::BlankSpace => editor.add_component(blank_space::Component::new()),
            Self::CurrentComparison => {
                editor.add_component(current_comparison::Component::new());
            }
            Self::CurrentPace => editor.add_component(current_pace::Component::new()),
            Self::Delta => editor.add_component(delta::Component::new()),
            Self::DetailedTimer => {
                editor.add_component(Box::new(detailed_timer::Component::new()));
            }
            Self::Graph => editor.add_component(graph::Component::new()),
            Self::PbChance => editor.add_component(pb_chance::Component::new()),
            Self::PossibleTimeSave => {
                editor.add_component(possible_time_save::Component::new());
            }
            Self::PreviousSegment => editor.add_component(previous_segment::Component::new()),
            Self::SegmentTime => editor.add_component(segment_time::Component::new()),
            Self::Separator => editor.add_component(separator::Component::new()),
            Self::Splits => {
                editor.add_component(splits::Component::new(livesplit_core::Lang::English));
            }
            Self::SumOfBest => editor.add_component(sum_of_best::Component::new()),
            Self::Text => editor.add_component(text::Component::new()),
            Self::Timer => editor.add_component(timer::Component::new()),
            Self::Title => editor.add_component(title::Component::new()),
            Self::TotalPlaytime => editor.add_component(total_playtime::Component::new()),
        }
    }
}

// ---------------------------------------------------------------------------
// Enum variant tables for ComboBox widgets
// ---------------------------------------------------------------------------

pub const ACCURACY_VARIANTS: &[(Accuracy, &str)] = &[
    (Accuracy::Seconds, "Seconds"),
    (Accuracy::Tenths, "Tenths"),
    (Accuracy::Hundredths, "Hundredths"),
    (Accuracy::Milliseconds, "Milliseconds"),
];

pub const DIGITS_FORMAT_VARIANTS: &[(DigitsFormat, &str)] = &[
    (DigitsFormat::SingleDigitSeconds, "1:23"),
    (DigitsFormat::DoubleDigitSeconds, "01:23"),
    (DigitsFormat::SingleDigitMinutes, "1:01:23"),
    (DigitsFormat::DoubleDigitMinutes, "01:01:23"),
    (DigitsFormat::SingleDigitHours, "1:01:01:23"),
    (DigitsFormat::DoubleDigitHours, "01:01:01:23"),
];

pub const ALIGNMENT_VARIANTS: &[(Alignment, &str)] = &[
    (Alignment::Auto, "Auto"),
    (Alignment::Left, "Left"),
    (Alignment::Center, "Center"),
];

pub const COLUMN_KIND_VARIANTS: &[(ColumnKind, &str)] = &[
    (ColumnKind::Time, "Time"),
    (ColumnKind::Variable, "Variable"),
];

pub const COLUMN_START_WITH_VARIANTS: &[(ColumnStartWith, &str)] = &[
    (ColumnStartWith::Empty, "Empty"),
    (ColumnStartWith::ComparisonTime, "Comparison Time"),
    (
        ColumnStartWith::ComparisonSegmentTime,
        "Comparison Segment Time",
    ),
    (ColumnStartWith::PossibleTimeSave, "Possible Time Save"),
];

pub const COLUMN_UPDATE_WITH_VARIANTS: &[(ColumnUpdateWith, &str)] = &[
    (ColumnUpdateWith::DontUpdate, "Don't Update"),
    (ColumnUpdateWith::SplitTime, "Split Time"),
    (ColumnUpdateWith::Delta, "Delta"),
    (ColumnUpdateWith::DeltaWithFallback, "Delta with Fallback"),
    (ColumnUpdateWith::SegmentTime, "Segment Time"),
    (ColumnUpdateWith::SegmentDelta, "Segment Delta"),
    (
        ColumnUpdateWith::SegmentDeltaWithFallback,
        "Segment Delta with Fallback",
    ),
];

pub const COLUMN_UPDATE_TRIGGER_VARIANTS: &[(ColumnUpdateTrigger, &str)] = &[
    (
        ColumnUpdateTrigger::OnStartingSegment,
        "On Starting Segment",
    ),
    (ColumnUpdateTrigger::Contextual, "Contextual"),
    (ColumnUpdateTrigger::OnEndingSegment, "On Ending Segment"),
];

pub const LAYOUT_DIRECTION_VARIANTS: &[(LayoutDirection, &str)] = &[
    (LayoutDirection::Vertical, "Vertical"),
    (LayoutDirection::Horizontal, "Horizontal"),
];

pub const FONT_STYLE_VARIANTS: &[(FontStyle, &str)] = &[
    (FontStyle::Normal, "Normal"),
    (FontStyle::Italic, "Italic"),
    (FontStyle::Oblique, "Oblique"),
];

pub const FONT_WEIGHT_VARIANTS: &[(FontWeight, &str)] = &[
    (FontWeight::Thin, "Thin"),
    (FontWeight::ExtraLight, "Extra Light"),
    (FontWeight::Light, "Light"),
    (FontWeight::SemiLight, "Semi Light"),
    (FontWeight::Normal, "Normal"),
    (FontWeight::Medium, "Medium"),
    (FontWeight::SemiBold, "Semi Bold"),
    (FontWeight::Bold, "Bold"),
    (FontWeight::ExtraBold, "Extra Bold"),
    (FontWeight::Black, "Black"),
    (FontWeight::ExtraBlack, "Extra Black"),
];

pub const FONT_STRETCH_VARIANTS: &[(FontStretch, &str)] = &[
    (FontStretch::UltraCondensed, "Ultra Condensed"),
    (FontStretch::ExtraCondensed, "Extra Condensed"),
    (FontStretch::Condensed, "Condensed"),
    (FontStretch::SemiCondensed, "Semi Condensed"),
    (FontStretch::Normal, "Normal"),
    (FontStretch::SemiExpanded, "Semi Expanded"),
    (FontStretch::Expanded, "Expanded"),
    (FontStretch::ExtraExpanded, "Extra Expanded"),
    (FontStretch::UltraExpanded, "Ultra Expanded"),
];

// ---------------------------------------------------------------------------
// LayoutEditorState
// ---------------------------------------------------------------------------

#[derive(PartialEq)]
pub(crate) enum EditorTab {
    Components,
    General,
}

pub(crate) struct LayoutEditorState {
    pub editor: livesplit_core::layout::editor::Editor,
    pub image_cache: ImageCache,
    pub active_tab: EditorTab,
    pub hotkey_capturing: Option<usize>,
}

impl LayoutEditorState {
    pub fn new(editor: livesplit_core::layout::editor::Editor) -> Self {
        Self {
            editor,
            image_cache: ImageCache::new(),
            active_tab: EditorTab::Components,
            hotkey_capturing: None,
        }
    }
}

// ---------------------------------------------------------------------------
// EditorAction
// ---------------------------------------------------------------------------

#[derive(PartialEq)]
enum EditorAction {
    None,
    Update,
    SaveToFile,
    Cancel,
}

// ---------------------------------------------------------------------------
// UI helpers
// ---------------------------------------------------------------------------

fn show_component_list(ui: &mut egui::Ui, snapshot: &EditorSnapshot) -> Option<usize> {
    let mut new_selection: Option<usize> = None;
    let selected = snapshot.selected_component as usize;

    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            for (i, name) in snapshot.components.iter().enumerate() {
                let is_selected = i == selected;
                let label = if is_selected {
                    format!("> {name}")
                } else {
                    format!("  {name}")
                };
                if ui.selectable_label(is_selected, label).clicked() && !is_selected {
                    new_selection = Some(i);
                }
            }
        });

    new_selection
}

fn show_component_buttons(
    ui: &mut egui::Ui,
    les: &mut LayoutEditorState,
    snapshot: &EditorSnapshot,
) {
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("add_component")
            .selected_text("Add...")
            .show_ui(ui, |ui| {
                for &kind in ComponentKind::ALL {
                    if ui.selectable_label(false, kind.display_name()).clicked() {
                        kind.add_to(&mut les.editor);
                    }
                }
            });
        ui.add_enabled_ui(snapshot.buttons.can_remove, |ui| {
            if ui.button("Remove").clicked() {
                les.editor.remove_component();
            }
        });
        ui.add_enabled_ui(snapshot.buttons.can_move_up, |ui| {
            if ui.button("Up").clicked() {
                les.editor.move_component_up();
            }
        });
        ui.add_enabled_ui(snapshot.buttons.can_move_down, |ui| {
            if ui.button("Down").clicked() {
                les.editor.move_component_down();
            }
        });
        if ui.button("Dup").clicked() {
            les.editor.duplicate_component();
        }
    });
}

/// Generic ComboBox for any Copy + PartialEq enum with a variants table.
fn enum_combobox<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash,
    current: T,
    variants: &[(T, &str)],
) -> Option<T> {
    let current_name = variants
        .iter()
        .find(|(v, _)| *v == current)
        .map(|(_, n)| *n)
        .unwrap_or("???");
    let mut result = None;
    egui::ComboBox::from_id_salt(id)
        .selected_text(current_name)
        .show_ui(ui, |ui| {
            for &(variant, name) in variants {
                if ui.selectable_label(variant == current, name).clicked() {
                    result = Some(variant);
                }
            }
        });
    result
}

/// Color picker button using egui's built-in RGBA color picker (f32 linear).
fn color_button(ui: &mut egui::Ui, color: &LsColor) -> Option<LsColor> {
    let mut rgba =
        egui::Rgba::from_rgba_unmultiplied(color.red, color.green, color.blue, color.alpha);
    let response = egui::color_picker::color_edit_button_rgba(
        ui,
        &mut rgba,
        egui::color_picker::Alpha::OnlyBlend,
    );
    if response.changed() {
        let [r, g, b, a] = rgba.to_rgba_unmultiplied();
        Some(LsColor::rgba(r, g, b, a))
    } else {
        None
    }
}

/// Widget for editing a Gradient value (ComboBox + 0-2 color buttons).
fn gradient_widget(
    ui: &mut egui::Ui,
    gradient: &Gradient,
    id: impl std::hash::Hash,
) -> Option<Gradient> {
    let variant_idx: usize = match gradient {
        Gradient::Transparent => 0,
        Gradient::Plain(_) => 1,
        Gradient::Vertical(_, _) => 2,
        Gradient::Horizontal(_, _) => 3,
    };
    let (mut c1, mut c2) = match gradient {
        Gradient::Transparent => (LsColor::transparent(), LsColor::transparent()),
        Gradient::Plain(c) => (*c, LsColor::transparent()),
        Gradient::Vertical(a, b) | Gradient::Horizontal(a, b) => (*a, *b),
    };
    let labels = ["Transparent", "Plain", "Vertical", "Horizontal"];
    let mut new_variant = variant_idx;
    let mut changed = false;

    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt(&id)
            .selected_text(labels[variant_idx])
            .show_ui(ui, |ui| {
                for (i, label) in labels.iter().enumerate() {
                    if ui.selectable_label(i == variant_idx, *label).clicked() {
                        new_variant = i;
                        changed = true;
                    }
                }
            });
        // Show color buttons based on variant
        if new_variant >= 1 {
            if let Some(c) = color_button(ui, &c1) {
                c1 = c;
                changed = true;
            }
        }
        if new_variant >= 2 {
            if let Some(c) = color_button(ui, &c2) {
                c2 = c;
                changed = true;
            }
        }
    });

    if changed {
        Some(match new_variant {
            0 => Gradient::Transparent,
            1 => Gradient::Plain(c1),
            2 => Gradient::Vertical(c1, c2),
            3 => Gradient::Horizontal(c1, c2),
            _ => unreachable!(),
        })
    } else {
        None
    }
}

/// Render an editable widget for a single settings field.
/// Returns `Some(new_value)` if the value changed, `None` otherwise.
fn show_field_widget(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    field: &livesplit_core::settings::Field,
    index: usize,
    hotkey_capturing: &mut Option<usize>,
) -> Option<Value> {
    match &field.value {
        Value::Bool(v) => {
            let mut val = *v;
            if ui.checkbox(&mut val, "").changed() {
                Some(Value::Bool(val))
            } else {
                None
            }
        }
        Value::UInt(v) => {
            let mut val = *v;
            if ui.add(egui::DragValue::new(&mut val)).changed() {
                Some(Value::UInt(val))
            } else {
                None
            }
        }
        Value::Int(v) => {
            let mut val = *v;
            if ui.add(egui::DragValue::new(&mut val)).changed() {
                Some(Value::Int(val))
            } else {
                None
            }
        }
        Value::String(v) => {
            let mut val = v.clone();
            if ui.text_edit_singleline(&mut val).changed() {
                Some(Value::String(val))
            } else {
                None
            }
        }
        Value::OptionalString(v) => {
            let mut enabled = v.is_some();
            let mut text = v.clone().unwrap_or_default();
            let mut changed = false;
            ui.horizontal(|ui| {
                if ui.checkbox(&mut enabled, "").changed() {
                    changed = true;
                }
                ui.add_enabled_ui(enabled, |ui| {
                    if ui.text_edit_singleline(&mut text).changed() {
                        changed = true;
                    }
                });
            });
            if changed {
                Some(Value::OptionalString(if enabled {
                    Some(text)
                } else {
                    None
                }))
            } else {
                None
            }
        }
        Value::Accuracy(v) => {
            enum_combobox(ui, ("accuracy", index), *v, ACCURACY_VARIANTS).map(Value::Accuracy)
        }
        Value::DigitsFormat(v) => {
            enum_combobox(ui, ("digits_fmt", index), *v, DIGITS_FORMAT_VARIANTS)
                .map(Value::DigitsFormat)
        }
        Value::OptionalTimingMethod(v) => {
            // Special case: Option<TimingMethod> with None meaning "current"
            let current_idx: usize = match v {
                None => 0,
                Some(TimingMethod::RealTime) => 1,
                Some(TimingMethod::GameTime) => 2,
            };
            let labels = ["Current Timing Method", "Real Time", "Game Time"];
            let mut result = None;
            egui::ComboBox::from_id_salt(("opt_timing", index))
                .selected_text(labels[current_idx])
                .show_ui(ui, |ui| {
                    for (i, label) in labels.iter().enumerate() {
                        if ui.selectable_label(i == current_idx, *label).clicked() {
                            result = Some(i);
                        }
                    }
                });
            result.map(|i| {
                Value::OptionalTimingMethod(match i {
                    0 => None,
                    1 => Some(TimingMethod::RealTime),
                    _ => Some(TimingMethod::GameTime),
                })
            })
        }
        Value::Alignment(v) => {
            enum_combobox(ui, ("alignment", index), *v, ALIGNMENT_VARIANTS).map(Value::Alignment)
        }
        Value::ColumnKind(v) => {
            enum_combobox(ui, ("col_kind", index), *v, COLUMN_KIND_VARIANTS).map(Value::ColumnKind)
        }
        Value::ColumnStartWith(v) => {
            enum_combobox(ui, ("col_start", index), *v, COLUMN_START_WITH_VARIANTS)
                .map(Value::ColumnStartWith)
        }
        Value::ColumnUpdateWith(v) => {
            enum_combobox(ui, ("col_update", index), *v, COLUMN_UPDATE_WITH_VARIANTS)
                .map(Value::ColumnUpdateWith)
        }
        Value::ColumnUpdateTrigger(v) => enum_combobox(
            ui,
            ("col_trigger", index),
            *v,
            COLUMN_UPDATE_TRIGGER_VARIANTS,
        )
        .map(Value::ColumnUpdateTrigger),
        Value::LayoutDirection(v) => {
            enum_combobox(ui, ("layout_dir", index), *v, LAYOUT_DIRECTION_VARIANTS)
                .map(Value::LayoutDirection)
        }
        Value::Color(c) => color_button(ui, c).map(Value::Color),
        Value::OptionalColor(oc) => {
            let mut enabled = oc.is_some();
            let mut color = oc.unwrap_or(LsColor::transparent());
            let mut changed = false;
            ui.horizontal(|ui| {
                if ui.checkbox(&mut enabled, "").changed() {
                    changed = true;
                }
                if enabled {
                    if let Some(c) = color_button(ui, &color) {
                        color = c;
                        changed = true;
                    }
                }
            });
            if changed {
                Some(Value::OptionalColor(if enabled {
                    Some(color)
                } else {
                    None
                }))
            } else {
                None
            }
        }
        Value::Gradient(g) => gradient_widget(ui, g, ("gradient", index)).map(Value::Gradient),
        Value::ListGradient(lg) => {
            let is_alternating = matches!(lg, ListGradient::Alternating(_, _));
            let variant_idx: usize = if is_alternating { 1 } else { 0 };
            let labels = ["Same", "Alternating"];
            let mut new_variant = variant_idx;
            let mut changed = false;
            let mut result_gradient = match lg {
                ListGradient::Same(g) => *g,
                ListGradient::Alternating(_, _) => Gradient::Transparent,
            };
            let (mut alt_c1, mut alt_c2) = match lg {
                ListGradient::Alternating(a, b) => (*a, *b),
                _ => (LsColor::transparent(), LsColor::transparent()),
            };

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt(("list_gradient", index))
                    .selected_text(labels[variant_idx])
                    .show_ui(ui, |ui| {
                        for (i, label) in labels.iter().enumerate() {
                            if ui.selectable_label(i == variant_idx, *label).clicked() {
                                new_variant = i;
                                changed = true;
                            }
                        }
                    });
                match new_variant {
                    0 => {
                        // Same: show gradient widget
                        if let Some(g) = gradient_widget(ui, &result_gradient, ("lg_grad", index)) {
                            result_gradient = g;
                            changed = true;
                        }
                    }
                    _ => {
                        // Alternating: show 2 color buttons
                        if let Some(c) = color_button(ui, &alt_c1) {
                            alt_c1 = c;
                            changed = true;
                        }
                        if let Some(c) = color_button(ui, &alt_c2) {
                            alt_c2 = c;
                            changed = true;
                        }
                    }
                }
            });
            if changed {
                Some(Value::ListGradient(match new_variant {
                    0 => ListGradient::Same(result_gradient),
                    _ => ListGradient::Alternating(alt_c1, alt_c2),
                }))
            } else {
                None
            }
        }
        Value::DeltaGradient(dg) => {
            // Flatten DeltaGradient into 7 options
            let flat_idx: usize = match dg {
                DeltaGradient::Gradient(Gradient::Transparent) => 0,
                DeltaGradient::Gradient(Gradient::Plain(_)) => 1,
                DeltaGradient::Gradient(Gradient::Vertical(_, _)) => 2,
                DeltaGradient::Gradient(Gradient::Horizontal(_, _)) => 3,
                DeltaGradient::DeltaPlain => 4,
                DeltaGradient::DeltaVertical => 5,
                DeltaGradient::DeltaHorizontal => 6,
            };
            let (mut c1, mut c2) = match dg {
                DeltaGradient::Gradient(Gradient::Plain(c)) => (*c, LsColor::transparent()),
                DeltaGradient::Gradient(Gradient::Vertical(a, b))
                | DeltaGradient::Gradient(Gradient::Horizontal(a, b)) => (*a, *b),
                _ => (LsColor::transparent(), LsColor::transparent()),
            };
            let labels = [
                "Transparent",
                "Plain",
                "Vertical",
                "Horizontal",
                "Delta Plain",
                "Delta Vertical",
                "Delta Horizontal",
            ];
            let mut new_idx = flat_idx;
            let mut changed = false;

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt(("delta_gradient", index))
                    .selected_text(labels[flat_idx])
                    .show_ui(ui, |ui| {
                        for (i, label) in labels.iter().enumerate() {
                            if ui.selectable_label(i == flat_idx, *label).clicked() {
                                new_idx = i;
                                changed = true;
                            }
                        }
                    });
                // Color buttons for gradient variants that need them
                if new_idx == 1 || new_idx == 2 || new_idx == 3 {
                    if let Some(c) = color_button(ui, &c1) {
                        c1 = c;
                        changed = true;
                    }
                }
                if new_idx == 2 || new_idx == 3 {
                    if let Some(c) = color_button(ui, &c2) {
                        c2 = c;
                        changed = true;
                    }
                }
            });
            if changed {
                Some(Value::DeltaGradient(match new_idx {
                    0 => DeltaGradient::Gradient(Gradient::Transparent),
                    1 => DeltaGradient::Gradient(Gradient::Plain(c1)),
                    2 => DeltaGradient::Gradient(Gradient::Vertical(c1, c2)),
                    3 => DeltaGradient::Gradient(Gradient::Horizontal(c1, c2)),
                    4 => DeltaGradient::DeltaPlain,
                    5 => DeltaGradient::DeltaVertical,
                    6 => DeltaGradient::DeltaHorizontal,
                    _ => unreachable!(),
                }))
            } else {
                None
            }
        }
        Value::LayoutBackground(lb) => match lb {
            LayoutBackground::Gradient(g) => gradient_widget(ui, g, ("layout_bg", index))
                .map(|g| Value::LayoutBackground(LayoutBackground::Gradient(g))),
            LayoutBackground::Image(bi) => {
                let mut brightness = bi.brightness;
                let mut opacity = bi.opacity;
                let mut blur = bi.blur;
                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Bright:");
                    if ui
                        .add(
                            egui::DragValue::new(&mut brightness)
                                .range(0.0..=1.0)
                                .speed(0.01),
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    ui.label("Opacity:");
                    if ui
                        .add(
                            egui::DragValue::new(&mut opacity)
                                .range(0.0..=1.0)
                                .speed(0.01),
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    ui.label("Blur:");
                    if ui
                        .add(egui::DragValue::new(&mut blur).range(0.0..=1.0).speed(0.01))
                        .changed()
                    {
                        changed = true;
                    }
                });
                if changed {
                    Some(Value::LayoutBackground(LayoutBackground::Image(
                        BackgroundImage {
                            image: bi.image,
                            brightness,
                            opacity,
                            blur,
                        },
                    )))
                } else {
                    None
                }
            }
        },
        Value::Font(opt_font) => {
            let mut enabled = opt_font.is_some();
            let mut font = opt_font.clone().unwrap_or_default();
            let mut changed = false;
            ui.vertical(|ui| {
                if ui.checkbox(&mut enabled, "Override").changed() {
                    changed = true;
                }
                if enabled {
                    if ui.text_edit_singleline(&mut font.family).changed() {
                        changed = true;
                    }
                    ui.horizontal(|ui| {
                        ui.label("Style:");
                        if let Some(s) = enum_combobox(
                            ui,
                            ("font_style", index),
                            font.style,
                            FONT_STYLE_VARIANTS,
                        ) {
                            font.style = s;
                            changed = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Weight:");
                        if let Some(w) = enum_combobox(
                            ui,
                            ("font_weight", index),
                            font.weight,
                            FONT_WEIGHT_VARIANTS,
                        ) {
                            font.weight = w;
                            changed = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Stretch:");
                        if let Some(s) = enum_combobox(
                            ui,
                            ("font_stretch", index),
                            font.stretch,
                            FONT_STRETCH_VARIANTS,
                        ) {
                            font.stretch = s;
                            changed = true;
                        }
                    });
                }
            });
            if changed {
                Some(Value::Font(if enabled { Some(font) } else { None }))
            } else {
                None
            }
        }
        Value::Hotkey(opt_hotkey) => {
            let is_capturing = *hotkey_capturing == Some(index);
            let mut result = None;
            if is_capturing {
                ui.horizontal(|ui| {
                    ui.label("Press a key...");
                    let events = ctx.input(|i| i.events.clone());
                    for event in &events {
                        if let egui::Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            ..
                        } = event
                        {
                            if *key == egui::Key::Escape {
                                *hotkey_capturing = None;
                                return;
                            }
                            let key_code = crate::hotkey::to_livesplit_keycode((*key).into());
                            let mods = crate::hotkey::to_livesplit_modifiers((*modifiers).into());
                            let hotkey = key_code.with_modifiers(mods);
                            result = Some(Value::Hotkey(Some(hotkey)));
                            *hotkey_capturing = None;
                            return;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        *hotkey_capturing = None;
                    }
                });
            } else {
                ui.horizontal(|ui| {
                    let display = match opt_hotkey {
                        Some(hk) => format!("{hk}"),
                        None => "(none)".to_owned(),
                    };
                    ui.label(display);
                    if ui.button("Set").clicked() {
                        *hotkey_capturing = Some(index);
                    }
                    if opt_hotkey.is_some() && ui.button("Clear").clicked() {
                        result = Some(Value::Hotkey(None));
                    }
                });
            }
            result
        }
    }
}

/// Render editable settings fields in a grid. Returns a list of (index, new_value) pairs
/// for fields that were changed this frame.
fn show_settings_editable(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    settings: &livesplit_core::settings::SettingsDescription,
    hotkey_capturing: &mut Option<usize>,
) -> Vec<(usize, Value)> {
    let mut changes = Vec::new();
    egui::Grid::new(ui.next_auto_id())
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            for (index, field) in settings.fields.iter().enumerate() {
                let label = ui.label(field.text.as_ref());
                if !field.tooltip.is_empty() {
                    label.on_hover_text(field.tooltip.as_ref());
                }
                if let Some(new_value) = show_field_widget(ui, ctx, field, index, hotkey_capturing)
                {
                    changes.push((index, new_value));
                }
                ui.end_row();
            }
        });
    changes
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

// ---------------------------------------------------------------------------
// Main viewport UI
// ---------------------------------------------------------------------------

fn layout_editor_ui(
    ctx: &egui::Context,
    state: &Mutex<Option<LayoutEditorState>>,
    timer: &SharedTimer,
    preview_slot: &Mutex<Option<LayoutState>>,
    actions: &Mutex<Vec<UiAction>>,
    open: &AtomicBool,
) {
    if ctx.input(|i| i.viewport().close_requested()) {
        open.store(false, Ordering::Relaxed);
        state.lock().take();
        return;
    }

    let mut guard = state.lock();
    let Some(ref mut les) = *guard else {
        return;
    };

    // Get editor snapshot (once per frame)
    let snapshot = les
        .editor
        .state(&mut les.image_cache, livesplit_core::Lang::English);

    let mut action = EditorAction::None;
    let mut new_selection: Option<usize> = None;
    let mut component_changes: Vec<(usize, Value)> = Vec::new();
    let mut general_changes: Vec<(usize, Value)> = Vec::new();

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Tab bar
            let prev_tab_is_components = les.active_tab == EditorTab::Components;
            ui.horizontal(|ui| {
                ui.selectable_value(&mut les.active_tab, EditorTab::Components, "Components");
                ui.selectable_value(&mut les.active_tab, EditorTab::General, "General Settings");
            });
            if prev_tab_is_components != (les.active_tab == EditorTab::Components) {
                les.hotkey_capturing = None;
            }
            ui.separator();

            match les.active_tab {
                EditorTab::Components => {
                    new_selection = show_component_list(ui, &snapshot);
                    show_component_buttons(ui, les, &snapshot);
                    ui.separator();
                    ui.strong(format!(
                        "Settings: {}",
                        &snapshot.components[snapshot.selected_component as usize]
                    ));
                    component_changes = show_settings_editable(
                        ui,
                        ctx,
                        &snapshot.component_settings,
                        &mut les.hotkey_capturing,
                    );
                }
                EditorTab::General => {
                    general_changes = show_settings_editable(
                        ui,
                        ctx,
                        &snapshot.general_settings,
                        &mut les.hotkey_capturing,
                    );
                }
            }

            ui.separator();
            action = show_action_buttons(ui);
        });
    });

    // Apply deferred selection
    if let Some(idx) = new_selection {
        les.editor.select(idx);
    }

    // Apply settings changes
    for (idx, val) in component_changes {
        les.editor.set_component_settings_value(idx, val);
    }
    {
        let editor = &mut les.editor;
        let image_cache = &les.image_cache;
        for (idx, val) in general_changes {
            editor.set_general_settings_value(idx, val, image_cache);
        }
    }

    // Keep this viewport repainting continuously so the preview stays fresh
    // (the timer is always ticking, so the layout state changes every frame).
    ctx.request_repaint();

    // Compute preview layout state for the main window
    if let Ok(timer_guard) = timer.read() {
        let snapshot = timer_guard.snapshot();
        let ls = les.editor.layout_state(
            &mut les.image_cache,
            &snapshot,
            livesplit_core::Lang::English,
        );
        *preview_slot.lock() = Some(ls);
        ctx.request_repaint_of(egui::ViewportId::ROOT);
    }

    // Handle update/save/cancel
    match action {
        EditorAction::Update => {
            if let Some(les) = guard.take() {
                let layout = les.editor.close();
                actions
                    .lock()
                    .push(UiAction::ApplyLayoutEdit(Box::new(layout)));
            }
            open.store(false, Ordering::Relaxed);
        }
        EditorAction::SaveToFile => {
            if let Some(les) = guard.take() {
                let layout = les.editor.close();
                let mut lock = actions.lock();
                lock.push(UiAction::ApplyLayoutEdit(Box::new(layout)));
                lock.push(UiAction::SaveLayoutDialog);
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

// ---------------------------------------------------------------------------
// Integration with LiveSplitCoreRenderer
// ---------------------------------------------------------------------------

impl LiveSplitCoreRenderer {
    pub(crate) fn show_layout_editor(&mut self, ctx: &egui::Context) {
        if !self.layout_editor_open.load(Ordering::Relaxed) {
            let mut guard = self.layout_editor_state.lock();
            if guard.is_some() {
                *guard = None;
            }
            return;
        }

        let state = self.layout_editor_state.clone();
        let timer = self.timer.clone();
        let preview_slot = self.layout_editor_preview.clone();
        let actions = self.ui_actions.clone();
        let open = self.layout_editor_open.clone();

        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of("layout_editor"),
            egui::ViewportBuilder::default()
                .with_title("Annelid Layout Editor")
                .with_inner_size([550.0, 560.0]),
            move |ctx, _class| {
                layout_editor_ui(ctx, &state, &timer, &preview_slot, &actions, &open);
            },
        );
    }
}
