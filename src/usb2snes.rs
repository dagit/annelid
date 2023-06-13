/*
 * Copyright (c) 2021 Sylvain "Skarsnik" Colinet
 *
 * This file is part of the usb2snes-cli project.
 * (see https://github.com/usb2snes/usb2snes-cli).
 *
 * usb2snes-cli is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * usb2snes-cli is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with QUsb2Snes.  If not, see <https://www.gnu.org/licenses/>.
 */

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::net::TcpStream;
use strum_macros::Display;
use tungstenite::protocol::WebSocket;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::Message;

#[derive(Display, Debug)]
#[allow(dead_code)]
pub enum Command {
    AppVersion,
    Name,
    DeviceList,
    Attach,
    Info,
    Boot,
    Reset,
    Menu,

    List,
    PutFile,
    GetFile,
    Rename,
    Remove,

    GetAddress,
}
#[derive(Display, Debug)]
#[allow(dead_code)]
pub enum Space {
    None,
    SNES,
    CMD,
}

#[derive(Debug)]
pub struct Infos {
    pub version: String,
    pub dev_type: String,
    pub game: String,
    pub flags: Vec<String>,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct USB2SnesQuery {
    Opcode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    Space: Option<String>,
    Flags: Vec<String>,
    Operands: Vec<String>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct USB2SnesResult {
    Results: Vec<String>,
}

#[derive(Eq, PartialEq)]
pub enum USB2SnesFileType {
    File = 0,
    Dir = 1,
}

pub struct USB2SnesFileInfo {
    pub name: String,
    pub file_type: USB2SnesFileType,
}

pub struct SyncClient {
    client: WebSocket<MaybeTlsStream<TcpStream>>,
    devel: bool,
}

impl SyncClient {
    pub fn connect() -> Result<SyncClient, Box<dyn Error>> {
        Ok(SyncClient {
            client: tungstenite::client::connect("ws://localhost:23074")?.0,
            devel: false,
        })
    }

    pub fn connect_with_devel() -> Result<SyncClient, Box<dyn Error>> {
        Ok(SyncClient {
            client: tungstenite::client::connect("ws://localhost:23074")?.0,
            devel: true,
        })
    }

    fn send_command(&mut self, command: Command, args: Vec<String>) -> Result<(), Box<dyn Error>> {
        self.send_command_with_space(command, None, args)
    }

    fn send_command_with_space(
        &mut self,
        command: Command,
        space: Option<Space>,
        args: Vec<String>,
    ) -> Result<(), Box<dyn Error>> {
        if self.devel {
            println!("Send command : {:?}", command);
        }
        let nspace = space.map(|sp| sp.to_string());
        let query = USB2SnesQuery {
            Opcode: command.to_string(),
            Space: nspace,
            Flags: vec![],
            Operands: args,
        };
        let json = serde_json::to_string_pretty(&query)?;
        if self.devel {
            println!("{}", json);
        }
        let message = Message::text(json);
        Ok(self.client.write_message(message)?)
    }

    fn get_reply(&mut self) -> Result<USB2SnesResult, Box<dyn Error>> {
        let reply = self.client.read_message()?;
        let mut textreply: String = String::from("");
        match reply {
            Message::Text(value) => {
                textreply = value;
            }
            _ => Err("Error getting a reply")?,
        };
        if self.devel {
            println!("Reply:");
            println!("{}", textreply);
        }
        Ok(serde_json::from_str(&textreply)?)
    }

    pub fn set_name(&mut self, name: String) -> Result<(), Box<dyn Error>> {
        self.send_command(Command::Name, vec![name])
    }

    pub fn app_version(&mut self) -> Result<String, Box<dyn Error>> {
        self.send_command(Command::AppVersion, vec![])?;
        let usbreply = self.get_reply()?;
        Ok(usbreply.Results[0].to_string())
    }

    pub fn list_device(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        self.send_command(Command::DeviceList, vec![])?;
        let usbreply = self.get_reply()?;
        Ok(usbreply.Results)
    }

    pub fn attach(&mut self, device: &String) -> Result<(), Box<dyn Error>> {
        self.send_command(Command::Attach, vec![device.to_string()])
    }

    pub fn info(&mut self) -> Result<Infos, Box<dyn Error>> {
        self.send_command(Command::Info, vec![])?;
        let usbreply = self.get_reply()?;
        let info: Vec<String> = usbreply.Results;
        Ok(Infos {
            version: info[0].clone(),
            dev_type: info[1].clone(),
            game: info[2].clone(),
            flags: (info[3..].to_vec()),
        })
    }

    pub fn reset(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_command(Command::Reset, vec![])
    }

    pub fn menu(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_command(Command::Menu, vec![])
    }

    pub fn boot(&mut self, toboot: String) -> Result<(), Box<dyn Error>> {
        self.send_command(Command::Boot, vec![toboot])
    }

    pub fn ls(&mut self, path: &String) -> Result<Vec<USB2SnesFileInfo>, Box<dyn Error>> {
        self.send_command(Command::List, vec![path.to_string()])?;
        let usbreply = self.get_reply()?;
        let vec_info = usbreply.Results;
        let mut toret: Vec<USB2SnesFileInfo> = vec![];
        let mut i = 0;
        while i < vec_info.len() {
            let info: USB2SnesFileInfo = USB2SnesFileInfo {
                file_type: if vec_info[i] == "1" {
                    USB2SnesFileType::File
                } else {
                    USB2SnesFileType::Dir
                },
                name: vec_info[i + 1].to_string(),
            };
            toret.push(info);
            i += 2;
        }
        Ok(toret)
    }

    pub fn send_file(&mut self, path: &String, data: Vec<u8>) -> Result<(), Box<dyn Error>> {
        self.send_command(
            Command::PutFile,
            vec![path.to_string(), format!("{:x}", data.len())],
        )?;
        let mut start = 0;
        let mut stop = 1024;
        while start < data.len() {
            self.client
                .write_message(Message::binary(&data[start..stop]))?;
            start += 1024;
            stop += 1024;
            if stop > data.len() {
                stop = data.len();
            }
        }
        Ok(())
    }

    pub fn get_file(&mut self, path: String) -> Result<Vec<u8>, Box<dyn Error>> {
        self.send_command(Command::GetFile, vec![path])?;
        let string_hex = self.get_reply()?.Results[0].to_string();
        let size = usize::from_str_radix(&string_hex, 16)?;
        let mut data: Vec<u8> = vec![];
        data.reserve(size);
        loop {
            let reply = self.client.read_message()?;
            match reply {
                Message::Binary(msgdata) => {
                    data.extend(&msgdata);
                }
                _ => Err("Error getting a reply")?,
            }
            if data.len() == size {
                break;
            }
        }
        Ok(data)
    }

    pub fn remove_path(&mut self, path: String) -> Result<(), Box<dyn Error>> {
        self.send_command(Command::Remove, vec![path])
    }

    pub fn get_address(&mut self, address: u32, size: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        self.send_command_with_space(
            Command::GetAddress,
            Some(Space::SNES),
            vec![format!("{:x}", address), format!("{:x}", size)],
        )?;
        let mut data: Vec<u8> = vec![];
        data.reserve(size);
        loop {
            let reply = self.client.read_message()?;
            match reply {
                Message::Binary(msgdata) => {
                    data.extend(&msgdata);
                }
                _ => Err("Error getting a reply")?,
            }
            if data.len() == size {
                break;
            }
        }
        Ok(data)
    }

    pub fn get_addresses(
        &mut self,
        pairs: &[(u32, usize)],
    ) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        let mut args = vec![];
        let mut total_size = 0;
        for &(address, size) in pairs.iter() {
            args.push(format!("{:x}", address));
            args.push(format!("{:x}", size));
            total_size += size;
        }
        self.send_command_with_space(Command::GetAddress, Some(Space::SNES), args)?;
        let mut data: Vec<u8> = vec![];
        let mut ret: Vec<Vec<u8>> = vec![];
        data.reserve(total_size);
        loop {
            let reply = self.client.read_message()?;
            match reply {
                Message::Binary(msgdata) => {
                    data.extend(&msgdata);
                }
                _ => Err("Error getting a reply")?,
            }
            if data.len() == total_size {
                break;
            }
        }
        let mut consumed = 0;
        for &(_address, size) in pairs.iter() {
            ret.push(data[consumed..consumed + size].into());
            consumed += size;
        }
        Ok(ret)
    }
}
