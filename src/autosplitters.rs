pub mod json;
pub mod supermetroid;

use anyhow::Result;
use livesplit_core::TimeSpan;

#[derive(Debug, Copy, Clone)]
pub struct SNESSummary {
    pub start: bool,
    pub reset: bool,
    pub split: bool,
}

pub trait AutoSplitter: Send {
    fn update(&mut self, client: &mut crate::usb2snes::SyncClient) -> Result<SNESSummary>;
    fn gametime_to_seconds(&self) -> Option<TimeSpan>;
    fn reset_game_tracking(&mut self);
}
