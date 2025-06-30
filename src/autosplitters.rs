pub mod json;
pub mod battletoads;
pub mod supermetroid;
use anyhow::Result;
use std::net::Ipv4Addr;
use livesplit_core::{GameTime, TimeSpan};

#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum AType {
    QUSB2SNES,
    NWA,
    ASL,
    CUSTOM,
}

// Not sure how to do this...
pub fn AutoSplitterSelector(game: &str,reset_timer_on_game_reset: bool) -> Object {
    match game {
        "Battletoads" => return battletoads::battletoadsAutoSplitter::new(
                            Ipv4Addr::new(0, 0, 0, 0),
                            48879, reset_timer_on_game_reset
                        ),
        &_ => todo!()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NWASummary {
    pub start: bool,
    pub reset: bool,
    pub split: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct SNESSummary {
    pub latency_average: f32,
    pub latency_stddev: f32,
    pub start: bool,
    pub reset: bool,
    pub split: bool,
}

pub trait AutoSplitter: Send {
    fn update(&mut self, client: &mut crate::usb2snes::SyncClient) -> Result<SNESSummary>;
    fn gametime_to_seconds(&self) -> Option<TimeSpan>;
    fn reset_game_tracking(&mut self);
}
