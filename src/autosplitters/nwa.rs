use crate::{autosplitters::NWASummary, config::app_config::AppConfig, nwa::NWASyncClient};
use anyhow::Result;
use std::{net::Ipv4Addr, sync::Arc};

pub mod battletoads;
pub mod supermetroid;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Game {
    Battletoads,
    SuperMetroid,
    // None,
}

pub fn nwaobject(game: Game, app_config: Arc<std::sync::RwLock<AppConfig>>) -> Box<dyn Splitter> {
    match game {
        Game::Battletoads => Box::new(battletoads::BattletoadsAutoSplitter {
            prior_level: 0,
            level: 0,
            reset_timer_on_game_reset: app_config
                .read()
                .unwrap()
                .reset_timer_on_game_reset
                .unwrap(),
            client: NWASyncClient::connect(&Ipv4Addr::new(0, 0, 0, 0).to_string(), 48879).unwrap(),
        }),
        Game::SuperMetroid => Box::new(supermetroid::SupermetroidAutoSplitter {
            prior_state: 0,
            state: 0,
            prior_room_id: 0,
            room_id: 0,
            reset_timer_on_game_reset: app_config
                .read()
                .unwrap()
                .reset_timer_on_game_reset
                .unwrap(),
            client: NWASyncClient::connect(&Ipv4Addr::new(0, 0, 0, 0).to_string(), 48879).unwrap(),
        }),
    }
}

pub trait Splitter {
    fn client_id(&mut self);

    fn emu_info(&mut self);

    fn emu_game_info(&mut self);

    fn emu_status(&mut self);

    fn core_info(&mut self);

    fn core_memories(&mut self);

    fn update(&mut self) -> Result<NWASummary>;

    fn start(&mut self) -> bool;

    fn reset(&mut self) -> bool;

    fn split(&mut self) -> bool;
}
