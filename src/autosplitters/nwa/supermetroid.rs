use crate::autosplitters::NWASummary;
use crate::nwa;
use anyhow::Result;
use std::net::Ipv4Addr;

pub enum Action {
    Start,
    Reset,
    Split,
}

pub struct SupermetroidAutoSplitter {
    // address: Ipv4Addr,
    // port: u32,
    prior_state: u8,
    state: u8,
    prior_room_id: u16,
    room_id: u16,
    reset_timer_on_game_reset: bool,
    client: nwa::NWASyncClient,
}

impl SupermetroidAutoSplitter {
    pub fn new(address: Ipv4Addr, port: u32, reset_timer_on_game_reset: bool) -> Self {
        SupermetroidAutoSplitter {
            // address,
            // port,
            prior_state: 0_u8,
            state: 0_u8,
            prior_room_id: 0_u16,
            room_id: 0_u16,
            reset_timer_on_game_reset,
            client: nwa::NWASyncClient::connect(&address.to_string(), port).unwrap(), // TODO: Need to handle error
        }
    }

    pub fn client_id(&mut self) {
        let cmd = "MY_NAME_IS";
        let args = Some("Annelid");
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{summary:#?}");
    }

    pub fn emu_info(&mut self) {
        let cmd = "EMULATOR_INFO";
        let args = Some("0");
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{summary:#?}");
    }

    pub fn emu_game_info(&mut self) {
        let cmd = "GAME_INFO";
        let args = None;
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{summary:#?}");
    }

    pub fn emu_status(&mut self) {
        let cmd = "EMULATION_STATUS";
        let args = None;
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{summary:#?}");
    }

    pub fn core_info(&mut self) {
        let cmd = "CORE_CURRENT_INFO";
        let args = None;
        let summary = self.client.execute_command(cmd, args).unwrap();
        println!("{summary:#?}");
    }

    pub fn core_memories(&mut self) {
        let cmd = "CORE_MEMORIES";
        let args = None;
        let summary = self.client.execute_command(cmd, args);
        println!("{summary:#?}");
    }

    pub fn update(&mut self) -> Result<NWASummary> {
        // read memory for the game state
        {
            self.prior_state = self.state;
            let cmd = "CORE_READ";
            let args = Some("WRAM;$0998;1");
            let summary = self.client.execute_command(cmd, args).unwrap();
            println!("{:#?}", summary);
            match summary {
                nwa::EmulatorReply::Binary(summary) => self.state = *summary.first().unwrap(),
                nwa::EmulatorReply::Error(summary) => println!("{:?}", summary),
                _ => println!("{summary:?}"),
            }
            println!("{:#?}", self.state);
        }

        // read memory for room
        {
            self.prior_room_id = self.room_id;
            let cmd = "CORE_READ";
            let args = Some("WRAM;$079B;2");
            let summary = self.client.execute_command(cmd, args).unwrap();
            println!("{summary:#?}");

            match summary {
                nwa::EmulatorReply::Binary(summary) => {
                    self.room_id =
                    // Have to reassemble the half word roomID 
                        ((*summary.last().unwrap() as u16) << 8) | *summary.first().unwrap() as u16
                }
                nwa::EmulatorReply::Error(summary) => println!("{:?}", summary),
                _ => println!("{summary:?}"),
            }
            println!("{:#?}", self.room_id);
        }

        // TODO: add the other memory reads

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
        self.state == 0x1F && self.prior_state == 0x1E
    }

    fn reset(&mut self) -> bool {
        self.room_id == 0 && self.prior_room_id != 0 && self.reset_timer_on_game_reset
    }

    fn split(&mut self) -> bool {
        self.room_id == 0xDF45 && self.prior_state == 0x8 && self.state == 0x20

        // TODO: add the rest of the splits
    }

    // pub fn set_address(&mut self, address: Ipv4Addr) {
        // self.address = address;
    // }

    // pub fn set_port(&mut self, port: u32) {
        // self.port = port;
    // }
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
