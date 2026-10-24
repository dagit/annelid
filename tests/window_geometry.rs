//! The window geometry saved with layouts, and when it counts as changed.
//!
//! A minimized window has no geometry: egui reports no inner or outer rect,
//! and Windows parks the window at (-32000, -32000). Closing annelid from the
//! taskbar while it was minimized used to compare the layout against a made-up
//! 800x600 window at (0, 0), so it always asked to save the layout, and saving
//! wrote that made-up geometry into the layout file.

use annelid::autosplitters::supermetroid::Settings;
use annelid::config::app_config::AppConfig;
use annelid::config::layout_meta::{LayoutMeta, WindowGeometry};
use annelid::livesplit_renderer::LiveSplitCoreRenderer;
use annelid::ui::file_dialogs::inject_layout_meta;
use eframe::egui;
use livesplit_core::{Lang, Layout, Run, Segment, Timer};
use std::io::Write;
use std::sync::Arc;

const TITLE_BAR_HEIGHT: f32 = 30.0;

/// A window whose frame's top-left corner is at (x, y) and whose client area
/// is width x height.
fn window_at(x: f32, y: f32, width: f32, height: f32) -> egui::ViewportInfo {
    egui::ViewportInfo {
        outer_rect: Some(egui::Rect::from_min_size(
            egui::pos2(x, y),
            egui::vec2(width, height + TITLE_BAR_HEIGHT),
        )),
        inner_rect: Some(egui::Rect::from_min_size(
            egui::pos2(x, y + TITLE_BAR_HEIGHT),
            egui::vec2(width, height),
        )),
        minimized: Some(false),
        ..Default::default()
    }
}

/// What egui reports for a minimized window.
fn minimized_window() -> egui::ViewportInfo {
    egui::ViewportInfo {
        minimized: Some(true),
        ..Default::default()
    }
}

fn content_rect(width: f32, height: f32) -> egui::Rect {
    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, height))
}

/// Runs one egui frame in which the root window looks like `window`.
fn run_frame(ctx: &egui::Context, window: egui::ViewportInfo, f: impl FnMut(&egui::Context)) {
    let mut input = egui::RawInput {
        // Like egui-winit: no screen rect while minimized.
        screen_rect: window
            .inner_rect
            .map(|r| content_rect(r.width(), r.height())),
        ..Default::default()
    };
    input.viewports.insert(egui::ViewportId::ROOT, window);
    let _ = ctx.run(input, f);
}

/// annelid with a layout whose saved window is at (100, 100) and 400x500.
fn annelid_with_layout(ctx: &egui::Context) -> (LiveSplitCoreRenderer, tempfile::NamedTempFile) {
    let mut run = Run::new();
    run.push_segment(Segment::new(""));
    let timer = Timer::new(run).expect("run has a segment").into_shared();
    let (sender, _receiver) = std::sync::mpsc::sync_channel(1);
    let mut app = LiveSplitCoreRenderer::new(
        timer,
        Layout::default_layout(Lang::English),
        Arc::new(parking_lot::RwLock::new(Settings::new())),
        sender,
        directories::ProjectDirs::from("", "", "annelid").expect("project dirs"),
        AppConfig::default(),
        Default::default(),
    );

    let mut layout =
        serde_json::to_value(Layout::default_layout(Lang::English).settings()).expect("serialize");
    let saved = LayoutMeta::from_geometry(WindowGeometry {
        x: 100.0,
        y: 100.0,
        width: 400.0,
        height: 500.0,
    });
    inject_layout_meta(&mut layout, &saved).expect("inject");
    let mut file = tempfile::NamedTempFile::with_suffix(".ls1l").expect("create temp");
    serde_json::to_writer(&mut file, &layout).expect("write");
    file.flush().expect("flush");

    let path = file.path().to_path_buf();
    run_frame(ctx, window_at(100.0, 100.0, 400.0, 500.0), |ctx| {
        app.load_layout(&path, ctx).expect("layout loads");
    });
    (app, file)
}

#[test]
fn geometry_is_outer_position_and_inner_size() {
    let meta = LayoutMeta::from_viewport(
        &window_at(100.0, 50.0, 400.0, 500.0),
        content_rect(400.0, 500.0),
        false,
    )
    .expect("a visible window has geometry");
    assert_eq!((meta.window_x, meta.window_y), (Some(100.0), Some(50.0)));
    assert_eq!(
        (meta.window_width, meta.window_height),
        (Some(400.0), Some(500.0))
    );
}

#[test]
fn minimized_window_has_no_geometry() {
    let meta = LayoutMeta::from_viewport(&minimized_window(), content_rect(400.0, 500.0), false);
    assert!(
        meta.is_none(),
        "a minimized window has no geometry, got {meta:?}"
    );
}

#[test]
fn wayland_window_size_comes_from_the_content_rect() {
    // Wayland can't report window positions, so egui never has an inner rect
    // there; the size is still known from the content rect.
    let wayland_window = egui::ViewportInfo {
        minimized: Some(false),
        ..Default::default()
    };
    let meta = LayoutMeta::from_viewport(&wayland_window, content_rect(400.0, 500.0), true)
        .expect("a Wayland window has a size");
    assert_eq!(
        (meta.window_width, meta.window_height),
        (Some(400.0), Some(500.0))
    );
    assert_eq!((meta.window_x, meta.window_y), (None, None));
}

#[test]
fn closing_while_minimized_does_not_ask_to_save_an_unchanged_layout() {
    let ctx = egui::Context::default();
    let (mut app, _layout) = annelid_with_layout(&ctx);
    run_frame(&ctx, window_at(100.0, 100.0, 400.0, 500.0), |ctx| {
        app.track_window_geometry(ctx)
    });
    // The user minimizes annelid, then closes it from the taskbar.
    run_frame(&ctx, minimized_window(), |ctx| {
        app.track_window_geometry(ctx)
    });

    assert!(
        !app.layout_needs_save(),
        "minimizing isn't a layout change; saving would write {:?}",
        app.window_geometry()
    );
}

#[test]
fn resizing_then_minimizing_saves_the_new_size() {
    let ctx = egui::Context::default();
    let (mut app, _layout) = annelid_with_layout(&ctx);
    run_frame(&ctx, window_at(100.0, 100.0, 600.0, 500.0), |ctx| {
        app.track_window_geometry(ctx)
    });
    run_frame(&ctx, minimized_window(), |ctx| {
        app.track_window_geometry(ctx)
    });

    assert!(app.layout_needs_save());
    let geometry = app.window_geometry().expect("the window had geometry");
    assert_eq!(
        (geometry.window_width, geometry.window_height),
        (Some(600.0), Some(500.0))
    );
}
