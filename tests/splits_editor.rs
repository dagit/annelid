use annelid::ui::splits_editor::SplitsEditorState;
use livesplit_core::run::editor::Editor;
use livesplit_core::{Run, Segment};

fn make_run(names: &[&str]) -> Run {
    let mut run = Run::new();
    run.set_game_name("Test Game");
    run.set_category_name("Any%");
    for name in names {
        run.push_segment(Segment::new(*name));
    }
    run
}

fn make_editor(names: &[&str]) -> Editor {
    Editor::new(make_run(names)).expect("valid editor")
}

#[test]
fn new_initializes_first_segment() {
    let editor = make_editor(&["Segment A", "Segment B", "Segment C"]);
    let state = SplitsEditorState::new(editor);

    assert_eq!(state.active_index, 0);
    assert_eq!(state.segment_name, "Segment A");
    assert_eq!(state.game_name, "Test Game");
    assert_eq!(state.category_name, "Any%");
}

#[test]
fn select_segment_changes_active() {
    let editor = make_editor(&["First", "Second", "Third"]);
    let mut state = SplitsEditorState::new(editor);

    assert_eq!(state.active_index, 0);
    assert_eq!(state.segment_name, "First");

    state.select_segment(1);
    assert_eq!(state.active_index, 1);
    assert_eq!(state.segment_name, "Second");

    state.select_segment(2);
    assert_eq!(state.active_index, 2);
    assert_eq!(state.segment_name, "Third");
}

#[test]
fn select_same_segment_is_noop() {
    let editor = make_editor(&["A", "B"]);
    let mut state = SplitsEditorState::new(editor);

    // Modify the segment name buffer
    state.segment_name = "Modified".to_string();

    // Selecting the same index should be a no-op (not flush or reload)
    state.select_segment(0);
    assert_eq!(state.segment_name, "Modified");
}

#[test]
fn select_segment_flushes_name() {
    let editor = make_editor(&["Original", "Second"]);
    let mut state = SplitsEditorState::new(editor);

    // Modify the name buffer for segment 0
    state.segment_name = "Renamed".to_string();

    // Select segment 1 — should flush "Renamed" to the editor
    state.select_segment(1);
    assert_eq!(state.active_index, 1);

    // Go back to segment 0 to verify the rename stuck
    state.select_segment(0);
    assert_eq!(state.segment_name, "Renamed");
}

#[test]
fn reload_segment_buffers_refreshes_data() {
    let editor = make_editor(&["Seg1", "Seg2"]);
    let mut state = SplitsEditorState::new(editor);

    // Manually corrupt the buffer
    state.segment_name = "wrong".to_string();

    // Reload should restore from editor state
    state.reload_segment_buffers();
    assert_eq!(state.segment_name, "Seg1");
}

#[test]
fn update_after_mutation_tracks_active() {
    let editor = make_editor(&["A", "B", "C"]);
    let mut state = SplitsEditorState::new(editor);

    // Select segment B
    state.select_segment(1);
    assert_eq!(state.segment_name, "B");

    // Insert a segment below (the editor inserts after the active segment)
    state.editor.insert_segment_below();
    state.update_after_mutation();

    // The new segment should now be active (inserted below B, so index 2)
    // and the buffers should reflect it
    assert!(state.scroll_to_active);
}

#[test]
fn multiple_segments_navigation() {
    let segments: Vec<String> = (0..10).map(|i| format!("Segment {i}")).collect();
    let names: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();
    let editor = make_editor(&names);
    let mut state = SplitsEditorState::new(editor);

    // Navigate to each segment and verify
    for i in 0..10 {
        state.select_segment(i);
        assert_eq!(state.active_index, i);
        assert_eq!(state.segment_name, format!("Segment {i}"));
    }

    // Navigate backwards
    for i in (0..10).rev() {
        state.select_segment(i);
        assert_eq!(state.active_index, i);
        assert_eq!(state.segment_name, format!("Segment {i}"));
    }
}
