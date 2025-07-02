use crate::autosplitters::NWASummary;
use crate::nwa;
use anyhow::Result;
use std::net::Ipv4Addr;

pub enum Action {
    start,
    reset,
    split,
}

pub struct battletoadsAutoSplitter {
    address: Ipv4Addr,
    port: u32,
    priorLevel: u8,
    level: u8,
    reset_timer_on_game_reset: bool,
    client: nwa::NWASyncClient,
}

impl battletoadsAutoSplitter {
    pub fn new(address: Ipv4Addr, port: u32, reset_timer_on_game_reset: bool) -> Self {
        battletoadsAutoSplitter {
            address,
            port,
            priorLevel: 0_u8,
            level: 0_u8,
            reset_timer_on_game_reset,
            client: nwa::NWASyncClient::connect(&address.to_string(), port).unwrap(), // TODO: Need to handle error
        }
    }

    pub fn clientID(&mut self) {
        let cmd = "MY_NAME_IS";
        let args = Some("Annelid");
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{:#?}", summary);
    }

    pub fn emuInfo(&mut self) {
        let cmd = "EMULATOR_INFO";
        let args = Some("0");
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{:#?}", summary);
    }

    pub fn emuGameInfo(&mut self) {
        let cmd = "GAME_INFO";
        let args = None;
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{:#?}", summary);
    }

    pub fn emuStatus(&mut self) {
        let cmd = "EMULATION_STATUS";
        let args = None;
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{:#?}", summary);
    }

    pub fn coreInfo(&mut self) {
        let cmd = "CORE_CURRENT_INFO";
        let args = None;
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{:#?}", summary);
    }

    pub fn coreMemories(&mut self) {
        let cmd = "CORE_MEMORIES";
        let args = None;
        let summary = self.client.execute_command(cmd, args);
        println!("{:#?}", summary);
    }

    pub fn update(&mut self) -> Result<NWASummary> {
        self.priorLevel = self.level;
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
        if self.level == 1 && self.priorLevel == 0 {
            return true;
        }
        false
    }

    fn reset(&mut self) -> bool {
        if self.level == 0 && self.priorLevel != 0 && self.reset_timer_on_game_reset {
            return true;
        }
        false
    }

    fn split(&mut self) -> bool {
        if self.level > self.priorLevel && self.priorLevel < 100 {
            return true;
        }
        false
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
