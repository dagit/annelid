use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::livesplit_renderer::LiveSplitCoreRenderer;
use crate::logging::LogBuffer;

fn log_viewer_ui(ctx: &egui::Context, log_buffer: &LogBuffer, open: &AtomicBool) {
    if ctx.input(|i| i.viewport().close_requested()) {
        open.store(false, Ordering::Relaxed);
        return;
    }

    // Request repaints so new log lines appear
    ctx.request_repaint_after(std::time::Duration::from_secs(1));

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                log_buffer.lock().clear();
            }
            let count = log_buffer.lock().len();
            ui.label(format!("{count} lines"));
        });
        ui.separator();

        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let buf = log_buffer.lock();
                for line in buf.iter() {
                    ui.label(line);
                }
            });
    });
}

impl LiveSplitCoreRenderer {
    pub(crate) fn show_log_viewer(&self, ctx: &egui::Context) {
        if !self.log_viewer_open.load(Ordering::Relaxed) {
            return;
        }

        let log_buffer = self.log_buffer.clone();
        let open = self.log_viewer_open.clone();

        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of("log_viewer"),
            egui::ViewportBuilder::default()
                .with_title("Annelid Log")
                .with_inner_size([600.0, 400.0]),
            move |ctx, _class| {
                log_viewer_ui(ctx, &log_buffer, &open);
            },
        );
    }
}
