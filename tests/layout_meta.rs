use annelid::config::layout_meta::{LayoutMeta, WindowGeometry};
use std::io::Write;

#[test]
fn from_geometry_stores_values() {
    let meta = LayoutMeta::from_geometry(WindowGeometry {
        x: 100.0,
        y: 200.0,
        width: 800.0,
        height: 600.0,
    });
    assert_eq!(meta.window_x, Some(100.0));
    assert_eq!(meta.window_y, Some(200.0));
    assert_eq!(meta.window_width, Some(800.0));
    assert_eq!(meta.window_height, Some(600.0));
}

#[test]
fn differs_from_within_tolerance() {
    let a = LayoutMeta::from_geometry(WindowGeometry {
        x: 100.0,
        y: 200.0,
        width: 800.0,
        height: 600.0,
    });
    let b = LayoutMeta::from_geometry(WindowGeometry {
        x: 101.0, // within 2.0 tolerance
        y: 200.5,
        width: 800.0,
        height: 601.5,
    });
    assert!(!a.differs_from(&b));
}

#[test]
fn differs_from_outside_tolerance() {
    let a = LayoutMeta::from_geometry(WindowGeometry {
        x: 100.0,
        y: 200.0,
        width: 800.0,
        height: 600.0,
    });
    let b = LayoutMeta::from_geometry(WindowGeometry {
        x: 100.0,
        y: 200.0,
        width: 810.0, // more than 2.0 difference
        height: 600.0,
    });
    assert!(a.differs_from(&b));
}

#[test]
fn differs_from_none_vs_some() {
    let a = LayoutMeta {
        window_x: Some(100.0),
        window_y: None,
        window_width: None,
        window_height: None,
    };
    let b = LayoutMeta::default();
    assert!(a.differs_from(&b));
}

#[test]
fn differs_from_both_none() {
    let a = LayoutMeta::default();
    let b = LayoutMeta::default();
    assert!(!a.differs_from(&b));
}

#[test]
fn from_layout_file_json() {
    let json = serde_json::json!({
        "annelid": {
            "window_x": 50.0,
            "window_y": 75.0,
            "window_width": 400.0,
            "window_height": 300.0
        }
    });
    let mut tmp = tempfile::NamedTempFile::with_suffix(".ls1l").expect("create temp file");
    write!(tmp, "{}", serde_json::to_string(&json).unwrap()).unwrap();
    tmp.flush().unwrap();

    let meta = LayoutMeta::from_layout_file(tmp.path()).expect("should parse JSON layout");
    assert_eq!(meta.window_x, Some(50.0));
    assert_eq!(meta.window_y, Some(75.0));
    assert_eq!(meta.window_width, Some(400.0));
    assert_eq!(meta.window_height, Some(300.0));
}

#[test]
fn from_layout_file_json_no_annelid_key() {
    let json = serde_json::json!({"other_key": "value"});
    let mut tmp = tempfile::NamedTempFile::with_suffix(".ls1l").expect("create temp file");
    write!(tmp, "{}", serde_json::to_string(&json).unwrap()).unwrap();
    tmp.flush().unwrap();

    assert!(LayoutMeta::from_layout_file(tmp.path()).is_none());
}

#[test]
fn from_layout_file_xml() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Layout>
  <Mode>Vertical</Mode>
  <X>120</X>
  <Y>240</Y>
  <VerticalWidth>350</VerticalWidth>
  <VerticalHeight>700</VerticalHeight>
</Layout>"#;
    let mut tmp = tempfile::NamedTempFile::with_suffix(".lsl").expect("create temp file");
    write!(tmp, "{}", xml).unwrap();
    tmp.flush().unwrap();

    let meta = LayoutMeta::from_layout_file(tmp.path()).expect("should parse XML layout");
    assert_eq!(meta.window_x, Some(120.0));
    assert_eq!(meta.window_y, Some(240.0));
    assert_eq!(meta.window_width, Some(350.0));
    assert_eq!(meta.window_height, Some(700.0));
}

#[test]
fn from_layout_file_xml_horizontal_mode() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Layout>
  <Mode>Horizontal</Mode>
  <X>50</X>
  <Y>60</Y>
  <HorizontalWidth>1024</HorizontalWidth>
  <HorizontalHeight>200</HorizontalHeight>
</Layout>"#;
    let mut tmp = tempfile::NamedTempFile::with_suffix(".lsl").expect("create temp file");
    write!(tmp, "{}", xml).unwrap();
    tmp.flush().unwrap();

    let meta = LayoutMeta::from_layout_file(tmp.path()).expect("should parse horizontal layout");
    assert_eq!(meta.window_width, Some(1024.0));
    assert_eq!(meta.window_height, Some(200.0));
}

#[test]
fn from_layout_file_nonexistent() {
    assert!(LayoutMeta::from_layout_file(std::path::Path::new("/nonexistent/file.lsl")).is_none());
}

#[test]
fn from_layout_file_invalid_content() {
    let mut tmp = tempfile::NamedTempFile::with_suffix(".ls1l").expect("create temp file");
    write!(tmp, "this is not json or xml").unwrap();
    tmp.flush().unwrap();

    assert!(LayoutMeta::from_layout_file(tmp.path()).is_none());
}
