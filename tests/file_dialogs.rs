use annelid::config::layout_meta::{LayoutMeta, WindowGeometry};
use annelid::ui::file_dialogs::inject_layout_meta;
use std::io::Write;

#[test]
fn inject_meta_adds_annelid_key() {
    let mut json = serde_json::json!({"some_setting": true});
    let meta = LayoutMeta::from_geometry(WindowGeometry {
        x: 100.0,
        y: 200.0,
        width: 800.0,
        height: 600.0,
    });
    inject_layout_meta(&mut json, &meta).expect("should inject");

    assert!(json.get("annelid").is_some(), "should have annelid key");
    let annelid = json.get("annelid").unwrap();
    assert_eq!(annelid.get("window_x").unwrap().as_f64().unwrap(), 100.0);
    assert_eq!(annelid.get("window_y").unwrap().as_f64().unwrap(), 200.0);
    assert_eq!(annelid.get("window_width").unwrap().as_f64().unwrap(), 800.0);
    assert_eq!(
        annelid.get("window_height").unwrap().as_f64().unwrap(),
        600.0
    );
}

#[test]
fn inject_meta_preserves_existing_keys() {
    let mut json = serde_json::json!({
        "direction": "Vertical",
        "components": []
    });
    let meta = LayoutMeta::from_geometry(WindowGeometry {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 300.0,
    });
    inject_layout_meta(&mut json, &meta).expect("should inject");

    // Existing keys should still be present
    assert_eq!(json.get("direction").unwrap().as_str().unwrap(), "Vertical");
    assert!(json.get("components").unwrap().is_array());
    // And annelid key should be added
    assert!(json.get("annelid").is_some());
}

#[test]
fn inject_meta_overwrites_existing_annelid_key() {
    let mut json = serde_json::json!({
        "annelid": {"old_data": true}
    });
    let meta = LayoutMeta::from_geometry(WindowGeometry {
        x: 50.0,
        y: 60.0,
        width: 1024.0,
        height: 768.0,
    });
    inject_layout_meta(&mut json, &meta).expect("should inject");

    let annelid = json.get("annelid").unwrap();
    // Should have new data, not old
    assert!(annelid.get("old_data").is_none());
    assert_eq!(annelid.get("window_width").unwrap().as_f64().unwrap(), 1024.0);
}

#[test]
fn inject_meta_with_none_fields() {
    let mut json = serde_json::json!({});
    let meta = LayoutMeta::default(); // all None
    inject_layout_meta(&mut json, &meta).expect("should inject");

    let annelid = json.get("annelid").unwrap();
    assert!(annelid.get("window_x").unwrap().is_null());
    assert!(annelid.get("window_y").unwrap().is_null());
}

#[test]
fn inject_meta_round_trips_with_from_layout_file() {
    let meta = LayoutMeta::from_geometry(WindowGeometry {
        x: 42.0,
        y: 84.0,
        width: 500.0,
        height: 350.0,
    });
    let mut json = serde_json::json!({"some_layout_data": true});
    inject_layout_meta(&mut json, &meta).expect("should inject");

    // Write to temp file and read back via LayoutMeta::from_layout_file
    let mut tmp = tempfile::NamedTempFile::with_suffix(".ls1l").expect("create temp");
    serde_json::to_writer(&mut tmp, &json).expect("write");
    tmp.flush().expect("flush");

    let read_back =
        LayoutMeta::from_layout_file(tmp.path()).expect("should read annelid metadata back");
    assert_eq!(read_back.window_x, Some(42.0));
    assert_eq!(read_back.window_y, Some(84.0));
    assert_eq!(read_back.window_width, Some(500.0));
    assert_eq!(read_back.window_height, Some(350.0));
}
