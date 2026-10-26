pub mod connection_log;
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
    /// What the game is doing, for the log, e.g. `game state 0x08, room 0x91f8`.
    fn game_status(&self) -> String;
}
