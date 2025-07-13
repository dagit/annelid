use crate::autosplitters::NWASummary;
use crate::nwa;
use anyhow::Result;
use std::net::Ipv4Addr;

pub enum Action {
    Start,
    Reset,
    Split,
}

pub struct BattletoadsAutoSplitter {
    address: Ipv4Addr,
    port: u32,
    prior_level: u8,
    level: u8,
    reset_timer_on_game_reset: bool,
    client: nwa::NWASyncClient,
}

impl BattletoadsAutoSplitter {
    pub fn new(address: Ipv4Addr, port: u32, reset_timer_on_game_reset: bool) -> Self {
        BattletoadsAutoSplitter {
            address,
            port,
            prior_level: 0_u8,
            level: 0_u8,
            reset_timer_on_game_reset,
            client: nwa::NWASyncClient::connect(&address.to_string(), port).unwrap(), // TODO: Need to handle error
        }
    }

    pub fn client_id(&mut self) {
        let cmd = "MY_NAME_IS";
        let args = Some("Annelid");
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{:#?}", summary);
    }

    pub fn emu_info(&mut self) {
        let cmd = "EMULATOR_INFO";
        let args = Some("0");
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{:#?}", summary);
    }

    pub fn emu_game_info(&mut self) {
        let cmd = "GAME_INFO";
        let args = None;
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{:#?}", summary);
    }

    pub fn emu_status(&mut self) {
        let cmd = "EMULATION_STATUS";
        let args = None;
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{:#?}", summary);
    }

    pub fn core_info(&mut self) {
        let cmd = "CORE_CURRENT_INFO";
        let args = None;
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{:#?}", summary);
    }

    pub fn core_memories(&mut self) {
        let cmd = "CORE_MEMORIES";
        let args = None;
        let summary = self.client.execute_command(cmd, args);
        println!("{:#?}", summary);
    }

    pub fn update(&mut self) -> Result<NWASummary> {
        self.prior_level = self.level;
        let cmd = "CORE_READ";
        let args = Some("RAM;$0010;1");
        let summary = self.client.execute_command(cmd, args).unwrap();
        // println!("{:#?}", summary);
        match summary {
            nwa::EmulatorReply::Binary(summary) => self.level = *summary.first().unwrap(),
            nwa::EmulatorReply::Error(summary) => println!("{:?}", summary),
            _ => println!("{:?}", summary),
        }
        // println!("{:#?}", level);

        let start = self.start();
        let reset = self.reset();
        let split = self.split();
        Ok(NWASummary {
            start,
            reset,
            split,
        })
    }

    fn start(&mut self) -> bool {
        if self.level == 1 && self.prior_level == 0 {
            return true;
        }
        false
    }

    fn reset(&mut self) -> bool {
        if self.level == 0 && self.prior_level != 0 && self.reset_timer_on_game_reset {
            return true;
        }
        false
    }

    fn split(&mut self) -> bool {
        if self.level > self.prior_level && self.prior_level < 100 {
            return true;
        }
        false
    }
    
    pub fn set_address(&mut self, address: Ipv4Addr) {
        self.address = address;
    }
    
    pub fn set_port(&mut self, port: u32) {
        self.port = port;
    }
}

// let cmd = "CORE_INFO";
// let args = Some("quickerNES");
// let summary = client.execute_command(cmd, args);
// println!("{:#?}",summary);

// let cmd = "CORES_LIST";
// let args = None;
// let summary = client.execute_command(cmd, args);
// println!("{:#?}",summary);

// let cmd = "LIST_BIZHAWK_DOMAINS";
// let args = None;
// let summary = client.execute_command(cmd, args);
// println!("{:#?}",summary);
