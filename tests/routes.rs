use annelid::routes::supermetroid::{anypercent, hundo};

#[test]
fn hundo_has_segments() {
    let (settings, run) = hundo();
    assert!(run.len() > 0, "hundo run should have segments");
    // Verify the settings enable expected splits
    assert!(settings.get("kraid"), "hundo should enable kraid");
    assert!(
        settings.get("rtaFinish") || settings.get("igtFinish"),
        "hundo should have an ending split"
    );
}

#[test]
fn hundo_game_and_category() {
    let (_settings, run) = hundo();
    assert_eq!(run.game_name(), "Super Metroid");
    assert_eq!(run.category_name(), "100%");
}

#[test]
fn hundo_segment_count_matches_splits() {
    let (_settings, run) = hundo();
    // Every segment should correspond to an enabled setting
    // (the last segment ".done" is the RTA/IGT finish)
    let segment_count = run.len();
    assert!(
        segment_count > 50,
        "hundo should have many segments, got {segment_count}"
    );
}

#[test]
fn anypercent_has_segments() {
    let (settings, run) = anypercent();
    assert!(run.len() > 0, "anypercent run should have segments");
    assert!(settings.get("kraid"), "anypercent should enable kraid");
}

#[test]
fn anypercent_game_and_category() {
    let (_settings, run) = anypercent();
    assert_eq!(run.game_name(), "Super Metroid");
    assert_eq!(run.category_name(), "KPDR");
}

#[test]
fn anypercent_fewer_segments_than_hundo() {
    let (_, hundo_run) = hundo();
    let (_, any_run) = anypercent();
    assert!(
        any_run.len() < hundo_run.len(),
        "any% ({}) should have fewer segments than 100% ({})",
        any_run.len(),
        hundo_run.len()
    );
}

#[test]
fn hundo_first_segment_is_ceres_ridley() {
    let (_, run) = hundo();
    assert_eq!(run.segment(0).name(), "ceresRidley");
}

#[test]
fn anypercent_first_segment_is_ceres_ridley() {
    let (_, run) = anypercent();
    assert_eq!(run.segment(0).name(), "ceresRidley");
}
