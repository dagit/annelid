use serde_derive::{Deserialize, Serialize};
use std::path::Path;

/// Window geometry metadata stored alongside layout files.
/// All values are in logical points (DPI-independent).
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct LayoutMeta {
    pub window_x: Option<f32>,
    pub window_y: Option<f32>,
    pub window_width: Option<f32>,
    pub window_height: Option<f32>,
}

/// Plain geometry values extracted from a window, independent of any GUI framework.
pub struct WindowGeometry {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutMeta {
    /// Build a LayoutMeta from plain geometry values.
    pub fn from_geometry(geo: WindowGeometry) -> Self {
        LayoutMeta {
            window_x: Some(geo.x),
            window_y: Some(geo.y),
            window_width: Some(geo.width),
            window_height: Some(geo.height),
        }
    }

    /// Try to extract window metadata from a layout file (.ls1l JSON or .lsl XML).
    /// Returns None if the file can't be read or doesn't contain window metadata.
    pub fn from_layout_file(path: &Path) -> Option<Self> {
        let contents = std::fs::read_to_string(path).ok()?;

        // Try JSON (.ls1l) — look for "annelid" key
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
            if let Some(annelid_val) = json.get("annelid") {
                if let Ok(meta) = serde_json::from_value::<LayoutMeta>(annelid_val.clone()) {
                    return Some(meta);
                }
            }
        }

        // Try XML (.lsl) — parse Layout element for window geometry
        if let Ok(doc) = roxmltree::Document::parse(&contents) {
            return Self::from_xml_doc(&doc);
        }

        None
    }

    /// Extract window metadata from a parsed XML layout document.
    fn from_xml_doc(doc: &roxmltree::Document) -> Option<Self> {
        use std::str::FromStr;
        for node in doc.root().children() {
            if node.tag_name().name() == "Layout" {
                let mut mode = None;
                let mut x = None;
                let mut y = None;
                let mut width = None;
                let mut height = None;
                node.children().for_each(|d| {
                    if d.tag_name().name() == "Mode" {
                        mode = d.text().map(|s| s.to_owned());
                    }
                    if d.tag_name().name() == "X" {
                        x = d.text().and_then(|d| f32::from_str(d).ok());
                    }
                    if d.tag_name().name() == "Y" {
                        y = d.text().and_then(|d| f32::from_str(d).ok());
                    }
                    if let Some(ref m) = mode {
                        if d.tag_name().name() == format!("{m}Width") {
                            width = d.text().and_then(|d| f32::from_str(d).ok());
                        }
                        if d.tag_name().name() == format!("{m}Height") {
                            height = d.text().and_then(|d| f32::from_str(d).ok());
                        }
                    }
                });
                if width.is_some() || height.is_some() || x.is_some() || y.is_some() {
                    return Some(LayoutMeta {
                        window_x: x,
                        window_y: y,
                        window_width: width,
                        window_height: height,
                    });
                }
            }
        }
        None
    }

    /// Returns true if `other` differs from `self` by more than a small
    /// tolerance (to ignore sub-pixel adjustments by the window manager).
    pub fn differs_from(&self, other: &LayoutMeta) -> bool {
        const TOLERANCE: f32 = 2.0;
        let differs = |a: Option<f32>, b: Option<f32>| -> bool {
            match (a, b) {
                (Some(a), Some(b)) => (a - b).abs() > TOLERANCE,
                (None, None) => false,
                _ => true,
            }
        };
        differs(self.window_width, other.window_width)
            || differs(self.window_height, other.window_height)
            || differs(self.window_x, other.window_x)
            || differs(self.window_y, other.window_y)
    }
}
