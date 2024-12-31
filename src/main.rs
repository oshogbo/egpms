use std::env;
use std::process;
use std::time::Duration;
use std::io;
use std::fs;

use rusb::UsbContext;
use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Config {
    sockets: Vec<Slot>,
}

#[derive(Deserialize)]
struct Slot {
    socket_id: u8,
    name: String,
}

const TIMEOUT: Duration = Duration::from_secs(1);

fn from_id_to_device(id: u8) -> u8 {
    0x03 * id
}

enum SetStatus {
    StatusEnable = 1,
    StatusDisable = 0,
}

impl SetStatus {
    fn exec(self, device: &rusb::DeviceHandle<rusb::Context>, socket_id: u8) {
        let devid = from_id_to_device(socket_id);
        let data = [devid, self as u8];
        let count = device
            .write_control(0x21, 0x09, 0x0300 + (devid as u16), 0x0, &data, TIMEOUT)
            .unwrap();
        println!("Done: {}", count);
    }
}

fn read_config() -> Result<Config, io::Error> {
    let filename = std::env::var("HOME").expect("HOME variable not set") + "/.egpms.toml";

    let content = match fs::read_to_string(filename) {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e),
    }?;

    if content == "" {
        return Ok(Config{sockets: vec![]});
    }

    toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

fn parse_socket_id(config: &Config, args: &[String]) -> Result<u8, &'static str> {
    if args.len() < 1 {
        return Err("not enough arguments");
    }

    let socket_id: u8;

    if let Ok(id) = args[0].parse::<u8>() {
        socket_id = id;
    } else if let Some(socket) = config.sockets.iter().find(|s| s.name == args[0]) {
        socket_id = socket.socket_id;
    } else {
        return Err("Provided socket ID is invalid");
    }

    if socket_id < 1 || socket_id > 4 {
        return Err("Unknwon socket");
    }

    Ok(socket_id)
}

trait ConfigCMD {
    fn run(&self, device: &rusb::DeviceHandle<rusb::Context>);
    fn parse(config : &Config, args: &[String]) -> Result<Box<dyn ConfigCMD>, &'static str>
    where
        Self: Sized;
}

struct ConfigEnable {
    socket_id: u8,
}

struct ConfigDisable {
    socket_id: u8,
}

struct ConfigStatus {
    socket_id: u8,
}

impl ConfigCMD for ConfigEnable {
    fn parse(config : &Config, args: &[String]) -> Result<Box<dyn ConfigCMD>, &'static str> {
        match parse_socket_id(config, args) {
            Ok(socket_id) => Ok(Box::new(Self { socket_id })),
            Err(err) => Err(err),
        }
    }

    fn run(&self, device: &rusb::DeviceHandle<rusb::Context>) {
        SetStatus::StatusEnable.exec(device, self.socket_id);
    }
}

impl ConfigCMD for ConfigDisable {
    fn parse(config : &Config, args: &[String]) -> Result<Box<dyn ConfigCMD>, &'static str> {
        match parse_socket_id(config, args) {
            Ok(socket_id) => Ok(Box::new(Self { socket_id })),
            Err(err) => Err(err),
        }
    }
    fn run(&self, device: &rusb::DeviceHandle<rusb::Context>) {
        SetStatus::StatusDisable.exec(device, self.socket_id);
    }
}

impl ConfigCMD for ConfigStatus {
    fn parse(config: &Config, args: &[String]) -> Result<Box<dyn ConfigCMD>, &'static str> {
        if args.len() == 0 {
            return Ok(Box::new(Self { socket_id: 0 }));
        }
        match parse_socket_id(config, args) {
            Ok(socket_id) => Ok(Box::new(Self { socket_id })),
            Err(err) => Err(err),
        }
    }
    fn run(&self, device: &rusb::DeviceHandle<rusb::Context>) {
        if self.socket_id != 0 {
            return self.run_one(device, self.socket_id);
        }
        for i in 1..=4 {
            self.run_one(device, i);
        }
    }
}
impl ConfigStatus {
    fn run_one(&self, device: &rusb::DeviceHandle<rusb::Context>, id: u8) {
        let mut data: [u8; 2] = [0x02, 0x00];
        let devid = from_id_to_device(id);
        device
            .read_control(0xa1, 0x01, 0x0300 + (devid as u16), 0x0, &mut data, TIMEOUT)
            .unwrap();
        println!(
            "Socket {} is {}",
            id,
            if data[1] == 0 { "offline" } else { "online" }
        );
    }
}

fn parse_cmd(config: &Config) -> Result<Box<dyn ConfigCMD>, &'static str> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Missing cmd");
    }
    match args[1].as_str() {
        "enable" => ConfigEnable::parse(&config, &args[2..]),
        "disable" => ConfigDisable::parse(&config, &args[2..]),
        "status" => ConfigStatus::parse(&config, &args[2..]),
        _ => Err("Unknwon option"),
    }
}

fn usage() -> ! {
    let args: Vec<String> = env::args().collect();
    println!("Usage: ");
    println!("{} status [socket_id/socket_name]", args[0]);
    println!("{} enable  socket_id/socket_name", args[0]);
    println!("{} disable socket_id/socket_name", args[0]);

    process::exit(1);
}

fn main() {
    let config = read_config().unwrap_or_else(|err| {
        println!("Error {}", err);
        println!("");
        process::exit(1);
    });

    let cmd = parse_cmd(&config).unwrap_or_else(|err| {
        println!("Error {}", err);
        println!("");
        usage();
    });

    let context = rusb::Context::new().unwrap();
    let device = context.open_device_with_vid_pid(0x04b4, 0xfd15).unwrap();

    cmd.run(&device);
}
