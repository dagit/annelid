// pub mod nwa {

use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
//use std::ptr::read;
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    InvalidError,
    InvalidCommand,
    InvalidArgument,
    NotAllowed,
    ProtocolError,
}
#[derive(Debug)]
pub struct NWAError {
    pub kind: ErrorKind,
    pub reason: String,
}

#[derive(Debug)]
pub enum AsciiReply {
    Ok,
    Hash(HashMap<String, String>),
    ListHash(Vec<HashMap<String, String>>),
}
#[derive(Debug)]
pub enum EmulatorReply {
    Ascii(AsciiReply),
    Error(NWAError),
    Binary(Vec<u8>),
}

#[derive(Debug)]
pub struct NWASyncClient {
    pub connection: TcpStream,
    pub port: u32,
    pub addr: SocketAddr,
}

impl NWASyncClient {
    pub fn connect(ip: &str, port: u32) -> Result<NWASyncClient, std::io::Error> {
        let addr: Vec<_> = format!("{}:{}", ip, port)
            .to_socket_addrs()
            .expect("Can't resolve address")
            .collect();
        //println!("{:?}", addr);
        let co = TcpStream::connect_timeout(&addr[0], Duration::from_millis(1000))?;
        Ok(NWASyncClient {
            connection: co,
            port,
            addr: addr[0],
        })
    }

    pub fn get_reply(&mut self) -> Result<EmulatorReply, std::io::Error> {
        let mut read_stream = BufReader::new(self.connection.try_clone().unwrap());
        let mut first_byte = [0_u8; 1];
        if read_stream.read(&mut first_byte)? == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "Read 0 byte",
            ));
        }
        let first_byte = first_byte[0];

        // Ascii
        if first_byte == b'\n' {
            let mut map: HashMap<String, String> = HashMap::new();
            let mut line: Vec<u8> = vec![];
            loop {
                line.clear();

                let rep = read_stream.read_until(b'\n', &mut line)?;
                //println!("{:?}", String::from_utf8(line.clone()));
                if line[0] == b'\n' && map.is_empty() {
                    return Ok(EmulatorReply::Ascii(AsciiReply::Ok));
                }
                if line[0] == b'\n' {
                    break;
                }
                if rep == 0 {
                    break;
                }
                let mut key = [0_u8; 100];
                let mut value = [0_u8; 1024];
                let mut cpt = 0;
                while line[cpt] != b':' && line[cpt] != b'\n' {
                    key[cpt] = line[cpt];
                    cpt += 1;
                }
                let end_key = cpt;
                // Should have stopped on :
                if line[cpt] == b'\n' {
                    return Err(std::io::Error::other("Mal formed reply"));
                }
                cpt += 1;
                let offset = cpt;
                while line[cpt] != b'\n' {
                    value[cpt - offset] = line[cpt];
                    cpt += 1;
                }
                let end_value = cpt - offset;
                map.insert(
                    String::from_utf8_lossy(&key[0..end_key]).to_string(),
                    String::from_utf8_lossy(&value[0..end_value]).to_string(),
                );
            }
            if map.contains_key("error") {
                if let Some(reason) = map.get("reason") {
                    let mkind: ErrorKind = match map.get("error").unwrap().as_str() {
                        "protocol_error" => ErrorKind::ProtocolError,
                        "invalid_command" => ErrorKind::InvalidCommand,
                        "invalid_argument" => ErrorKind::InvalidArgument,
                        "not_allowed" => ErrorKind::NotAllowed,
                        _ => ErrorKind::InvalidError,
                    };
                    return Ok(EmulatorReply::Error(NWAError {
                        kind: mkind,
                        reason: reason.to_string(),
                    }));
                } else {
                    return Ok(EmulatorReply::Error(NWAError {
                        kind: ErrorKind::InvalidError,
                        reason: String::from("Invalid reason"),
                    }));
                }
            }
            return Ok(EmulatorReply::Ascii(AsciiReply::Hash(map)));
        }
        if first_byte == 0 {
            let mut header = vec![0; 4];
            let _r_size = read_stream.read(&mut header)?;
            println!();
            //println!("Reading {:}", r_size);
            //println!("Header : {:?}", header);
            let header = header;
            let mut size: u32;
            size = (header[0] as u32) << 24;
            size += (header[1] as u32) << 16;
            size += (header[2] as u32) << 8;
            size += header[3] as u32;
            //println!("Size : {:}", size);
            let msize = size as usize;
            let mut data: Vec<u8> = vec![0; msize];
            //println!("Size : {:}", size);
            read_stream.read_exact(&mut data)?;
            //println!("Size : {:}", size);
            return Ok(EmulatorReply::Binary(data));
        }
        Err(std::io::Error::other("Invalid reply"))
    }

    pub fn execute_command(
        &mut self,
        cmd: &str,
        arg_string: Option<&str>,
    ) -> Result<EmulatorReply, std::io::Error> {
        if arg_string.is_none() {
            self.connection.write_all(format!("{}\n", cmd).as_bytes())?;
        } else {
            self.connection
                .write_all(format!("{} {}\n", cmd, arg_string.unwrap()).as_bytes())?;
        }
        self.get_reply()
    }

    pub fn execute_raw_command(&mut self, cmd: &str, arg_string: Option<&str>) {
        if arg_string.is_none() {
            self.connection.write_all(format!("{}\n", cmd).as_bytes());
        } else {
            self.connection
                .write_all(format!("{} {}\n", cmd, arg_string.unwrap()).as_bytes());
        }
    }

    pub fn send_data(&mut self, data: Vec<u8>) {
        let mut buf: Vec<u8> = vec![0; 5];
        let size = data.len();
        buf[0] = 0;
        buf[1] = ((size >> 24) & 0xFF) as u8;
        buf[2] = ((size >> 16) & 0xFF) as u8;
        buf[3] = ((size >> 8) & 0xFF) as u8;
        buf[4] = (size & 0xFF) as u8;
        self.connection.write_all(&buf);
        self.connection.write_all(&data);
    }
    pub fn is_connected(&mut self) -> bool {
        let mut buf = vec![0; 0];
        if let Ok(_usize) = self.connection.peek(&mut buf) {
            return true;
        }
        false
    }

    pub fn close(&mut self) {
        self.connection.shutdown(Shutdown::Both);
    }
    pub fn reconnected(&mut self) -> Result<bool, std::io::Error> {
        self.connection = TcpStream::connect_timeout(&self.addr, Duration::from_millis(1000))?;
        Ok(true)
    }
}

// }
