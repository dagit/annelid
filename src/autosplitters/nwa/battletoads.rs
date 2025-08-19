use crate::autosplitters::nwa::Splitter;
use crate::autosplitters::NWASummary;
use crate::config::app_config::YesOrNo;
use crate::nwa;
use anyhow::Result;

pub struct BattletoadsAutoSplitter {
    pub prior_level: u8,
    pub level: u8,
    pub reset_timer_on_game_reset: YesOrNo,
    pub client: nwa::NWASyncClient,
}

impl Splitter for BattletoadsAutoSplitter {
    fn client_id(&mut self) {
        let cmd = "MY_NAME_IS";
        let args = Some("Annelid");
        self.client.execute_command(cmd, args).unwrap();
        // println!("{summary:#?}");
    }

    fn emu_info(&mut self) {
        let cmd = "EMULATOR_INFO";
        let args = Some("0");
        self.client.execute_command(cmd, args).unwrap();
        // println!("{summary:#?}");
    }

    fn emu_game_info(&mut self) {
        let cmd = "GAME_INFO";
        let args = None;
        self.client.execute_command(cmd, args).unwrap();
        // println!("{summary:#?}");
    }

    fn emu_status(&mut self) {
        let cmd = "EMULATION_STATUS";
        let args = None;
        self.client.execute_command(cmd, args).unwrap();
        // println!("{summary:#?}");
    }

    fn core_info(&mut self) {
        let cmd = "CORE_CURRENT_INFO";
        let args = None;
        self.client.execute_command(cmd, args).unwrap();
        // println!("{summary:#?}");
    }

    fn core_memories(&mut self) {
        let cmd = "CORE_MEMORIES";
        let args = None;
        let _ = self.client.execute_command(cmd, args);
        // println!("{summary:#?}");
    }

    fn update(&mut self) -> Result<NWASummary> {
        self.prior_level = self.level;
        let cmd = "CORE_READ";
        let args = Some("RAM;$0010;1");
        let summary = self.client.execute_command(cmd, args).unwrap();
        // println!("{:#?}", summary);
        match summary {
            nwa::EmulatorReply::Binary(summary) => self.level = *summary.first().unwrap(),
            nwa::EmulatorReply::Error(summary) => println!("{summary:?}"),
            _ => println!("{summary:?}"),
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
        if self.level == 0
            && self.prior_level != 0
            && self.reset_timer_on_game_reset == YesOrNo::Yes
        {
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
