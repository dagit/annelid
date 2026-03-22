use annelid::ui::layout_editor::{
    ComponentKind, ACCURACY_VARIANTS, ALIGNMENT_VARIANTS, COLUMN_KIND_VARIANTS,
    COLUMN_START_WITH_VARIANTS, COLUMN_UPDATE_TRIGGER_VARIANTS, COLUMN_UPDATE_WITH_VARIANTS,
    DIGITS_FORMAT_VARIANTS, FONT_STRETCH_VARIANTS, FONT_STYLE_VARIANTS, FONT_WEIGHT_VARIANTS,
    LAYOUT_DIRECTION_VARIANTS,
};

#[test]
fn all_component_kinds_have_display_names() {
    for kind in ComponentKind::ALL {
        let name = kind.display_name();
        assert!(!name.is_empty(), "{kind:?} has empty display name");
    }
}

#[test]
fn component_kind_all_count() {
    // There are 17 component kinds
    assert_eq!(ComponentKind::ALL.len(), 17);
}

#[test]
fn display_names_are_unique() {
    let names: Vec<&str> = ComponentKind::ALL.iter().map(|k| k.display_name()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        names.len(),
        sorted.len(),
        "Duplicate display names found"
    );
}

#[test]
fn known_display_names() {
    assert_eq!(ComponentKind::Timer.display_name(), "Timer");
    assert_eq!(ComponentKind::Splits.display_name(), "Splits");
    assert_eq!(ComponentKind::Title.display_name(), "Title");
    assert_eq!(ComponentKind::PbChance.display_name(), "PB Chance");
    assert_eq!(
        ComponentKind::PossibleTimeSave.display_name(),
        "Possible Time Save"
    );
}

// --- Variant table completeness ---
// These tests verify that variant tables have no duplicates and have labels.

fn check_variant_table<T>(table: &[(T, &str)], name: &str) {
    assert!(!table.is_empty(), "{name} table is empty");
    for (i, (_, label)) in table.iter().enumerate() {
        assert!(
            !label.is_empty(),
            "{name}[{i}] has empty label"
        );
    }
    // Check labels are unique
    let labels: Vec<&str> = table.iter().map(|(_, l)| *l).collect();
    let mut sorted = labels.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        labels.len(),
        sorted.len(),
        "{name} has duplicate labels"
    );
}

#[test]
fn accuracy_variants_valid() {
    check_variant_table(ACCURACY_VARIANTS, "ACCURACY_VARIANTS");
    assert_eq!(ACCURACY_VARIANTS.len(), 4);
}

#[test]
fn digits_format_variants_valid() {
    check_variant_table(DIGITS_FORMAT_VARIANTS, "DIGITS_FORMAT_VARIANTS");
    assert_eq!(DIGITS_FORMAT_VARIANTS.len(), 6);
}

#[test]
fn alignment_variants_valid() {
    check_variant_table(ALIGNMENT_VARIANTS, "ALIGNMENT_VARIANTS");
}

#[test]
fn column_kind_variants_valid() {
    check_variant_table(COLUMN_KIND_VARIANTS, "COLUMN_KIND_VARIANTS");
}

#[test]
fn column_start_with_variants_valid() {
    check_variant_table(COLUMN_START_WITH_VARIANTS, "COLUMN_START_WITH_VARIANTS");
}

#[test]
fn column_update_with_variants_valid() {
    check_variant_table(COLUMN_UPDATE_WITH_VARIANTS, "COLUMN_UPDATE_WITH_VARIANTS");
}

#[test]
fn column_update_trigger_variants_valid() {
    check_variant_table(
        COLUMN_UPDATE_TRIGGER_VARIANTS,
        "COLUMN_UPDATE_TRIGGER_VARIANTS",
    );
}

#[test]
fn layout_direction_variants_valid() {
    check_variant_table(LAYOUT_DIRECTION_VARIANTS, "LAYOUT_DIRECTION_VARIANTS");
    assert_eq!(LAYOUT_DIRECTION_VARIANTS.len(), 2);
}

#[test]
fn font_style_variants_valid() {
    check_variant_table(FONT_STYLE_VARIANTS, "FONT_STYLE_VARIANTS");
}

#[test]
fn font_weight_variants_valid() {
    check_variant_table(FONT_WEIGHT_VARIANTS, "FONT_WEIGHT_VARIANTS");
    assert_eq!(FONT_WEIGHT_VARIANTS.len(), 11);
}

#[test]
fn font_stretch_variants_valid() {
    check_variant_table(FONT_STRETCH_VARIANTS, "FONT_STRETCH_VARIANTS");
    assert_eq!(FONT_STRETCH_VARIANTS.len(), 9);
}
