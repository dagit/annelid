use crate::autosplitters::supermetroid::Settings;
use crate::autosplitters::supermetroid::SuperMetroidAutoSplitter;
use crate::autosplitters::AutoSplitter;
use anyhow::{anyhow, Context};
use eframe::egui;
use livesplit_core::{Layout, SharedTimer};
use livesplit_hotkey::Hook;
use parking_lot::RwLock;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use thread_priority::ThreadBuilder;
use tracing::{span, Level};

use crate::config::app_config::*;
use crate::utils::*;
use crate::widget::glow_canvas::*;

pub enum ThreadEvent {
    TimerReset,
}

/// UI panel management state, separated from domain state for cleaner framework porting.
pub(crate) struct UiState {
    pub control_panel_open: Arc<AtomicBool>,
    pub ui_actions: Arc<parking_lot::Mutex<Vec<crate::ui::control_panel::UiAction>>>,
    pub settings_panel_open: Arc<AtomicBool>,
    pub settings_panel_state:
        Arc<parking_lot::Mutex<Option<crate::ui::app_settings::SettingsState>>>,
    pub splits_editor_open: Arc<AtomicBool>,
    pub splits_editor_state:
        Arc<parking_lot::Mutex<Option<crate::ui::splits_editor::SplitsEditorState>>>,
    pub layout_editor_open: Arc<AtomicBool>,
    pub layout_editor_state:
        Arc<parking_lot::Mutex<Option<crate::ui::layout_editor::LayoutEditorState>>>,
    pub layout_editor_preview: Arc<parking_lot::Mutex<Option<livesplit_core::layout::LayoutState>>>,
    pub log_viewer_open: Arc<AtomicBool>,
    pub show_settings_editor: Arc<AtomicBool>,
    pub autosplitter_settings_snapshot: Arc<parking_lot::Mutex<Option<Settings>>>,
    pub splits_editor_preview: Arc<parking_lot::Mutex<Option<livesplit_core::Run>>>,
    pub layout_modified: bool,
}

impl UiState {
    fn new() -> Self {
        UiState {
            control_panel_open: Arc::new(AtomicBool::new(false)),
            ui_actions: Arc::new(parking_lot::Mutex::new(Vec::new())),
            settings_panel_open: Arc::new(AtomicBool::new(false)),
            settings_panel_state: Arc::new(parking_lot::Mutex::new(None)),
            splits_editor_open: Arc::new(AtomicBool::new(false)),
            splits_editor_state: Arc::new(parking_lot::Mutex::new(None)),
            layout_editor_open: Arc::new(AtomicBool::new(false)),
            layout_editor_state: Arc::new(parking_lot::Mutex::new(None)),
            layout_editor_preview: Arc::new(parking_lot::Mutex::new(None)),
            log_viewer_open: Arc::new(AtomicBool::new(false)),
            show_settings_editor: Arc::new(AtomicBool::new(false)),
            autosplitter_settings_snapshot: Arc::new(parking_lot::Mutex::new(None)),
            splits_editor_preview: Arc::new(parking_lot::Mutex::new(None)),
            layout_modified: false,
        }
    }
}

pub struct LiveSplitCoreRenderer {
    pub(crate) layout: Layout,
    pub(crate) renderer: livesplit_core::rendering::software::BorrowedRenderer,
    pub(crate) gpu_renderer: Arc<parking_lot::Mutex<Option<livesplit_renderer_gpu::GlowRenderer>>>,
    pub(crate) layout_state: Arc<parking_lot::RwLock<Option<livesplit_core::layout::LayoutState>>>,
    pub(crate) image_cache: Arc<parking_lot::RwLock<livesplit_core::settings::ImageCache>>,
    pub(crate) timer: SharedTimer,
    pub(crate) settings: Arc<RwLock<Settings>>,
    pub(crate) can_exit: bool,
    pub(crate) is_exiting: bool,
    pub(crate) thread_chan: std::sync::mpsc::SyncSender<ThreadEvent>,
    pub(crate) project_dirs: directories::ProjectDirs,
    pub app_config: Arc<RwLock<AppConfig>>,
    pub(crate) app_config_processed: bool,
    pub(crate) glow_canvas: GlowCanvas,
    pub(crate) global_hotkey_hook: Option<Hook>,
    pub(crate) load_errors: Vec<anyhow::Error>,
    /// The window geometry that was last loaded from or saved to a layout file.
    /// Used to detect whether the user has moved/resized the window.
    pub(crate) saved_layout_meta: Option<crate::config::layout_meta::LayoutMeta>,
    pub(crate) log_buffer: crate::logging::LogBuffer,
    pub(crate) ui: UiState,
}

impl LiveSplitCoreRenderer {
    pub fn new(
        timer: SharedTimer,
        layout: Layout,
        settings: Arc<RwLock<Settings>>,
        chan: std::sync::mpsc::SyncSender<ThreadEvent>,
        project_dirs: directories::ProjectDirs,
        cli_config: AppConfig,
        log_buffer: crate::logging::LogBuffer,
    ) -> Self {
        LiveSplitCoreRenderer {
            timer,
            layout,
            renderer: livesplit_core::rendering::software::BorrowedRenderer::new(),
            gpu_renderer: Arc::new(parking_lot::Mutex::new(None)),
            image_cache: Arc::new(parking_lot::RwLock::new(
                livesplit_core::settings::ImageCache::new(),
            )),
            layout_state: Arc::new(parking_lot::RwLock::new(None)),
            settings,
            can_exit: false,
            is_exiting: false,
            thread_chan: chan,
            project_dirs,
            app_config: Arc::new(RwLock::new(cli_config)),
            app_config_processed: false,
            glow_canvas: GlowCanvas::new(),
            global_hotkey_hook: None,
            load_errors: vec![],
            saved_layout_meta: None,
            log_buffer,
            ui: UiState::new(),
        }
    }
}

impl eframe::App for LiveSplitCoreRenderer {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        if self.app_config.read().transparent_window == Some(YesOrNo::Yes) {
            [0.0, 0.0, 0.0, 0.0]
        } else {
            [0.0, 0.0, 0.0, 1.0]
        }
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let _frame = span!(Level::TRACE, "frame").entered();
        //let update_timer = std::time::Instant::now();
        if self.app_config_processed && !self.load_errors.is_empty() {
            let mut queue: Vec<anyhow::Error> = vec![];
            std::mem::swap(&mut queue, &mut self.load_errors);
            for e in queue.into_iter() {
                messagebox_on_error(move || Err(e))
            }
        }
        if !self.app_config_processed {
            self.process_app_config(ctx);
            self.app_config_processed = true;
        }
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested {
            self.is_exiting = true;
            if let Some(gl) = frame.gl() {
                if let Err(e) = self.confirm_save(gl, ctx) {
                    tracing::error!("Error during save on exit: {e}");
                }
            } else {
                tracing::error!("No GL context available during exit");
            }
            self.save_app_config();
        }
        if self.can_exit {
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Close);
            return;
        } else {
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::CancelClose)
        }
        let viewport = ctx.input(|i| i.content_rect());
        // Update layout state (shared by both renderers)
        {
            let _span = span!(Level::TRACE, "layout_state_update").entered();
            let layout_editor_open = self
                .ui
                .layout_editor_open
                .load(std::sync::atomic::Ordering::Relaxed);
            let splits_editor_open = self
                .ui
                .splits_editor_open
                .load(std::sync::atomic::Ordering::Relaxed);

            // If the layout editor is open, use its preview (computed in the
            // editor's deferred viewport callback — no editor lock needed here).
            let layout_preview = if layout_editor_open {
                self.ui.layout_editor_preview.lock().take()
            } else {
                // Editor just closed — clear any stale preview
                let stale = self.ui.layout_editor_preview.lock().take();
                if stale.is_some() {
                    *self.layout_state.write() = None;
                }
                None
            };

            // If the splits editor is open, use its preview run to compute
            // a layout state with the edited segment names/times.
            let splits_preview = if splits_editor_open {
                self.ui.splits_editor_preview.lock().take()
            } else {
                self.ui.splits_editor_preview.lock().take(); // drain stale
                None
            };

            if let Some(ls) = layout_preview {
                *self.layout_state.write() = Some(ls);
            } else if let Some(run) = splits_preview {
                if let Ok(preview_timer) = livesplit_core::Timer::new(run) {
                    let snapshot = preview_timer.snapshot();
                    let mut image_cache = self.image_cache.write();
                    let mut layout_state = self.layout_state.write();
                    *layout_state = Some(self.layout.state(
                        &mut image_cache,
                        &snapshot,
                        livesplit_core::Lang::English,
                    ));
                }
            } else if !layout_editor_open && !splits_editor_open {
                // Normal path: compute from self.layout
                let Ok(timer) = self.timer.read() else {
                    return;
                };
                let snapshot = timer.snapshot();
                let mut image_cache = self.image_cache.write();
                let mut layout_state = self.layout_state.write();
                match layout_state.as_mut() {
                    None => {
                        *layout_state = Some(self.layout.state(
                            &mut image_cache,
                            &snapshot,
                            livesplit_core::Lang::English,
                        ));
                    }
                    Some(ls) => {
                        self.layout.update_state(
                            ls,
                            &mut image_cache,
                            &snapshot,
                            livesplit_core::Lang::English,
                        );
                    }
                }
            }
            // else: editor is open but no preview yet (first frame) — keep
            // showing the stale layout_state until the editor produces one
        }

        if self.app_config.read().renderer == Some(RendererType::Gpu) {
            let ppp = ctx.input(|i| i.pixels_per_point());
            let width = (viewport.width() * ppp) as u32;
            let height = (viewport.height() * ppp) as u32;
            if width > 0 && height > 0 {
                let gpu = self.gpu_renderer.clone();
                let ls = self.layout_state.clone();
                let ic = self.image_cache.clone();
                let painter = ctx.layer_painter(egui::LayerId::background());
                painter.add(egui::PaintCallback {
                    rect: viewport,
                    callback: std::sync::Arc::new(egui_glow::CallbackFn::new(
                        move |_info, _painter| {
                            let _span = span!(Level::TRACE, "gpu_render").entered();
                            let ls_guard = ls.read();
                            let ic_guard = ic.read();
                            if let Some(layout_state) = ls_guard.as_ref() {
                                let mut gpu_guard = gpu.lock();
                                if let Some(gpu_renderer) = gpu_guard.as_mut() {
                                    // Always draw the layout background — its
                                    // own alpha controls OBS compositing.
                                    // Window-level transparency is handled by
                                    // clear_color() returning alpha=0.
                                    unsafe {
                                        gpu_renderer.render(
                                            layout_state,
                                            &ic_guard,
                                            [width, height],
                                            true,
                                        );
                                    }
                                }
                            }
                        },
                    )),
                });
            }
        } else {
            {
                let _span = span!(Level::TRACE, "update_frame_buffer").entered();
                let layout_state = self.layout_state.read();
                let image_cache = self.image_cache.read();
                let Some(gl) = frame.gl() else {
                    return;
                };
                self.glow_canvas
                    .update_frame_buffer(viewport, gl, |frame_buffer, sz, stride| {
                        if let Some(layout_state) = layout_state.as_ref() {
                            let _renderer_render_span =
                                span!(Level::TRACE, "renderer.render").entered();
                            self.renderer.render(
                                layout_state,
                                &image_cache,
                                frame_buffer,
                                sz,
                                stride,
                                true,
                            );
                        }
                    });
            }
            {
                let _span = span!(Level::TRACE, "paint_layer").entered();
                self.glow_canvas
                    .paint_layer(ctx, egui::LayerId::background(), viewport);
            }
        }
        let response = egui::Area::new("livesplit".into())
            .enabled(
                !self
                    .ui
                    .show_settings_editor
                    .load(std::sync::atomic::Ordering::Relaxed),
            )
            .movable(false)
            .show(ctx, |ui| {
                ui.set_width(ctx.input(|i| i.content_rect().width()));
                ui.set_height(ctx.input(|i| i.content_rect().height()));
            })
            .response;
        if response.secondary_clicked() {
            self.ui
                .control_panel_open
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }
        self.show_autosplitter_settings_window(ctx);
        self.show_control_panel(ctx);
        self.show_app_settings(ctx);
        self.show_splits_editor(ctx);
        self.show_layout_editor(ctx);
        self.show_log_viewer(ctx);
        self.process_ui_actions(ctx);
        ctx.input(|i| {
            let scroll_delta = i.raw_scroll_delta;
            if scroll_delta.y > 0.0 {
                self.layout.scroll_up();
            } else if scroll_delta.y < 0.0 {
                self.layout.scroll_down();
            }
        });
        self.handle_local_hotkeys(ctx);

        //println!("Time to update: {}μs", update_timer.elapsed().as_micros());
    }
}

fn panic_payload_to_string(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

pub fn app_init(
    app: &mut LiveSplitCoreRenderer,
    sync_receiver: std::sync::mpsc::Receiver<ThreadEvent>,
    cc: &eframe::CreationContext,
) {
    let context = cc.egui_ctx.clone();
    context.set_visuals(egui::Visuals::dark());
    app.load_app_config();
    if app.app_config.read().renderer == Some(RendererType::Gpu) {
        let gl = cc
            .gl
            .as_ref()
            .expect("eframe glow backend required for GPU renderer");
        match unsafe { livesplit_renderer_gpu::GlowRenderer::new(gl.clone()) } {
            Ok(gpu_renderer) => {
                *app.gpu_renderer.lock() = Some(gpu_renderer);
                tracing::info!("GPU renderer initialized");
            }
            Err(e) => {
                tracing::warn!("Failed to initialize GPU renderer, falling back to software: {e}");
                app.app_config.write().renderer = Some(RendererType::Software);
            }
        }
    }
    if app.app_config.read().global_hotkeys == Some(YesOrNo::Yes) {
        messagebox_on_error(|| app.enable_global_hotkeys());
    }
    let frame_rate = app
        .app_config
        .read()
        .frame_rate
        .unwrap_or(DEFAULT_FRAME_RATE);
    let polling_rate = app
        .app_config
        .read()
        .polling_rate
        .unwrap_or(DEFAULT_POLLING_RATE);
    // This thread is essentially just a refresh rate timer
    // it ensures that the gui thread is redrawn at the requested frame_rate,
    // possibly more often.
    match ThreadBuilder::default()
        .name("Frame Rate Thread".to_owned())
        .spawn(move |_| {
            if let Err(panic) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| loop {
                if frame_rate > 0.0 {
                    context.request_repaint_of(egui::ViewportId::ROOT);
                    std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / frame_rate));
                }
            })) {
                let msg = panic_payload_to_string(&panic);
                tracing::error!("Frame rate thread panicked: {msg}");
            }
        }) {
        Ok(_) => {}
        Err(e) => tracing::error!("Failed to spawn frame rate thread: {e}"),
    }

    // The timer, settings, and app_config are all behind
    // something equivalent to Arc<RwLock<_>> so it's safe
    // to clone them and pass the clone between threads.
    let timer = app.timer.clone();
    let settings = app.settings.clone();
    let app_config = app.app_config.clone();
    // This thread deals with polling the SNES at a fixed rate.
    if app_config.read().use_autosplitter == Some(YesOrNo::Yes) {
        match ThreadBuilder::default()
            .name("SNES Polling Thread".to_owned())
            // We could change this thread priority, but we probably
            // should leave it at the default to make sure we get timely
            // polling of SNES state
            .spawn(move |_| {
                if let Err(panic) =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        if polling_rate > 0.0 {
                            loop {
                                let period = std::time::Duration::from_secs_f32(1.0 / polling_rate);
                                print_on_error(|| -> anyhow::Result<()> {
                                    let mut client = crate::usb2snes::SyncClient::connect()
                                        .context("creating usb2snes connection")?;
                                    client.set_name("annelid")?;
                                    tracing::info!("Server version is {:?}", client.app_version()?);
                                    let mut devices = client.list_device()?.to_vec();
                                    if devices.len() != 1 {
                                        if devices.is_empty() {
                                            Err(anyhow!("No devices present"))?;
                                        } else {
                                            Err(anyhow!(
                                                "You need to select a device: {:#?}",
                                                devices
                                            ))?;
                                        }
                                    }
                                    let device =
                                        devices.pop().ok_or(anyhow!("Device list was empty"))?;
                                    tracing::info!("Using device: {device}");
                                    client.attach(&device)?;
                                    tracing::info!("Connected.");
                                    tracing::debug!("{:#?}", client.info()?);
                                    let mut autosplitter: Box<dyn AutoSplitter> =
                                        Box::new(SuperMetroidAutoSplitter::new(settings.clone()));
                                    let mut next = std::time::Instant::now() + period;
                                    loop {
                                        let summary = autosplitter.update(&mut client)?;
                                        if summary.start {
                                            timer
                                        .write()
                                        .map_err(|e| {
                                            anyhow!("failed to acquire write lock on timer: {e}")
                                        })?
                                        .start()
                                        .ok();
                                        }
                                        if summary.reset
                                            && app_config.read().reset_timer_on_game_reset
                                                == Some(YesOrNo::Yes)
                                        {
                                            timer
                                        .write()
                                        .map_err(|e| {
                                            anyhow!("failed to acquire write lock on timer: {e}")
                                        })?
                                        .reset(true)
                                        .ok();
                                        }
                                        if summary.split {
                                            if let Some(t) = autosplitter.gametime_to_seconds() {
                                                timer
                                                    .write()
                                                    .map_err(|e| {
                                                        anyhow!(
                                                    "failed to acquire write lock on timer: {e}"
                                                )
                                                    })?
                                                    .set_game_time(t)
                                                    .ok();
                                            }
                                            timer
                                        .write()
                                        .map_err(|e| {
                                            anyhow!("failed to acquire write lock on timer: {e}")
                                        })?
                                        .split()
                                        .ok();
                                        }
                                        // If the timer gets reset, we need to make a fresh snes state
                                        if let Ok(ThreadEvent::TimerReset) =
                                            sync_receiver.try_recv()
                                        {
                                            autosplitter.reset_game_tracking();
                                            //Reset the snes
                                            if app_config.read().reset_game_on_timer_reset
                                                == Some(YesOrNo::Yes)
                                            {
                                                client.reset()?;
                                            }
                                        }
                                        let now = std::time::Instant::now();
                                        if now < next {
                                            std::thread::sleep(next - now);
                                            next += period;
                                        } else {
                                            // skip sleep; we are late
                                            next = now + period;
                                        }
                                    }
                                });
                                std::thread::sleep(std::time::Duration::from_millis(1000));
                            }
                        }
                    }))
                {
                    let msg = panic_payload_to_string(&panic);
                    tracing::error!("SNES polling thread panicked: {msg}");
                }
            }) {
            Ok(_) => {}
            Err(e) => tracing::error!("Failed to spawn SNES polling thread: {e}"),
        }
    }
}
