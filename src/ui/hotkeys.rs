use anyhow::{anyhow, Result};

use crate::config::app_config::*;
use crate::hotkey::*;
use crate::livesplit_renderer::{LiveSplitCoreRenderer, ThreadEvent};
use livesplit_hotkey::Hook;

impl LiveSplitCoreRenderer {
    pub fn enable_global_hotkeys(&mut self) -> Result<()> {
        // It would be more elegant to use get_or_insert_with, however
        // the `with` branch cannot have a `Result` type if we do that.
        let hook: &Hook = match self.global_hotkey_hook.as_ref() {
            None => {
                self.global_hotkey_hook = Some(Hook::new()?);
                self.global_hotkey_hook.as_ref().unwrap() // We just set it so this will always
                                                          // succeed.
            }
            Some(h) => h,
        };

        // This is a bit of a mess but it lets us reduce a lot of duplication.
        // the idea here is that make_cb gives us a fresh callback each time
        // we clone it. That way we can register the call back twice,
        // once for the primary key and once for the alternate key.
        fn reg<F>(hook: &Hook, hot_key: &HotKey, make_cb: F) -> Result<()>
        where
            F: Fn() + Send + 'static + Clone,
        {
            // main binding
            hook.register(hot_key.to_livesplit_hotkey(), make_cb.clone())?;
            // optional "alt" binding
            if let Some(alt_code) = to_livesplit_keycode_alternative(&hot_key.key) {
                let alt = livesplit_hotkey::Hotkey {
                    key_code: alt_code,
                    modifiers: to_livesplit_modifiers(&hot_key.modifiers),
                };
                hook.register(alt, make_cb)?;
            }
            Ok(())
        }

        let cfg = self
            .app_config
            .read()
            .map_err(|e| anyhow!("failed to read config: {e}"))?;
        let timer = self.timer.clone();
        let thread_chan = self.thread_chan.clone();
        let app_cfg = self.app_config.clone();

        tracing::debug!("Registering global hotkeys...");
        if let Some(hk) = cfg.hot_key_start {
            reg(hook, &hk, {
                let timer = timer.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.split_or_start().ok())
                        .map_err(|e| tracing::warn!("split/start lock failed: {e}"));
                }
            })?;
        }
        if let Some(hk) = cfg.hot_key_reset {
            reg(hook, &hk, {
                let timer = timer.clone();
                let tc = thread_chan.clone();
                let app_cfg = app_cfg.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.reset(true).ok())
                        .map_err(|e| tracing::warn!("reset lock failed: {e}"));
                    if app_cfg
                        .read()
                        .map(|g| g.use_autosplitter == Some(YesOrNo::Yes))
                        .unwrap_or(false)
                    {
                        tc.try_send(ThreadEvent::TimerReset).unwrap_or(());
                    }
                }
            })?;
        }
        if let Some(hk) = cfg.hot_key_undo {
            reg(hook, &hk, {
                let timer = timer.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.undo_split().ok())
                        .map_err(|e| tracing::warn!("undo lock failed: {e}"));
                }
            })?;
        }
        if let Some(hk) = cfg.hot_key_skip {
            reg(hook, &hk, {
                let timer = timer.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.skip_split().ok())
                        .map_err(|e| tracing::warn!("skip split lock failed: {e}"));
                }
            })?;
        }
        if let Some(hk) = cfg.hot_key_pause {
            reg(hook, &hk, {
                let timer = timer.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.toggle_pause().ok())
                        .map_err(|e| tracing::warn!("toggle pause lock failed: {e}"));
                }
            })?;
        }
        if let Some(hk) = cfg.hot_key_comparison_next {
            reg(hook, &hk, {
                let timer = timer.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.switch_to_next_comparison())
                        .map_err(|e| tracing::warn!("next comparison lock failed: {e}"));
                }
            })?;
        }
        if let Some(hk) = cfg.hot_key_comparison_prev {
            reg(hook, &hk, {
                let timer = timer.clone();
                move || {
                    let _ = timer
                        .write()
                        .map(|mut g| g.switch_to_previous_comparison())
                        .map_err(|e| tracing::warn!("prev comparison lock failed: {e}"));
                }
            })?;
        }

        tracing::debug!("Global hotkeys registered");
        Ok(())
    }

    pub(crate) fn handle_local_hotkeys(&mut self, ctx: &eframe::egui::Context) {
        let config = self.app_config.read().unwrap();
        if config.global_hotkeys != Some(YesOrNo::Yes) {
            ctx.input_mut(|input| {
                if let Some(hot_key) = config.hot_key_start {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().split_or_start().ok();
                    }
                }
                if let Some(hot_key) = config.hot_key_reset {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().reset(true).ok();
                        if config.use_autosplitter == Some(YesOrNo::Yes) {
                            self.thread_chan
                                .try_send(ThreadEvent::TimerReset)
                                .unwrap_or(());
                        }
                    }
                }
                if let Some(hot_key) = config.hot_key_undo {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().undo_split().ok();
                    }
                }
                if let Some(hot_key) = config.hot_key_skip {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().skip_split().ok();
                    }
                }
                if let Some(hot_key) = config.hot_key_pause {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().toggle_pause().ok();
                    }
                }
                if let Some(hot_key) = config.hot_key_comparison_next {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().switch_to_next_comparison();
                    }
                }
                if let Some(hot_key) = config.hot_key_comparison_prev {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().switch_to_previous_comparison();
                    }
                }
            });
        }
    }
}
