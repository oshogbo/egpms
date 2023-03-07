extern crate libusb;

use std::env;
use std::process;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(1);

fn from_id_to_device(id: u8) -> u8 {
    0x03 * id
}

enum SetStatus {
    StatusStart = 1,
    StatusStop = 0,
}

impl SetStatus {
    fn exec(self, device: &libusb::DeviceHandle, socket_id: u8) {
        let devid = from_id_to_device(socket_id);
        let data = [devid, self as u8];
        let count = device
            .write_control(0x21, 0x09, 0x0300 + (devid as u16), 0x0, &data, TIMEOUT)
            .unwrap();
        println!("Done: {}", count);
    }
}

fn parse_socket_id(args: &[String]) -> Result<u8, &'static str> {
    if args.len() < 1 {
        return Err("not enough arguments");
    }
    let socket_id: u8 = match args[0].parse() {
        Ok(num) => num,
        Err(_) => return Err("Socket id has to be number"),
    };
    if socket_id < 1 || socket_id > 4 {
        return Err("Unknwon socket");
    }

    Ok(socket_id)
}

trait ConfigCMD {
    fn run(&self, device: &libusb::DeviceHandle);
    fn parse(args: &[String]) -> Result<Box<dyn ConfigCMD>, &'static str>
    where
        Self: Sized;
}

struct ConfigStart {
    socket_id: u8,
}

struct ConfigStop {
    socket_id: u8,
}

struct ConfigStatus {
    socket_id: u8,
}

impl ConfigCMD for ConfigStart {
    fn parse(args: &[String]) -> Result<Box<dyn ConfigCMD>, &'static str> {
        match parse_socket_id(args) {
            Ok(socket_id) => Ok(Box::new(Self { socket_id })),
            Err(err) => Err(err),
        }
    }

    fn run(&self, device: &libusb::DeviceHandle) {
        SetStatus::StatusStart.exec(device, self.socket_id);
    }
}

impl ConfigCMD for ConfigStop {
    fn parse(args: &[String]) -> Result<Box<dyn ConfigCMD>, &'static str> {
        match parse_socket_id(args) {
            Ok(socket_id) => Ok(Box::new(Self { socket_id })),
            Err(err) => Err(err),
        }
    }
    fn run(&self, device: &libusb::DeviceHandle) {
        SetStatus::StatusStop.exec(device, self.socket_id);
    }
}

impl ConfigCMD for ConfigStatus {
    fn parse(args: &[String]) -> Result<Box<dyn ConfigCMD>, &'static str> {
        if args.len() == 0 {
            return Ok(Box::new(Self { socket_id: 0 }));
        }
        match parse_socket_id(args) {
            Ok(socket_id) => Ok(Box::new(Self { socket_id })),
            Err(err) => Err(err),
        }
    }
    fn run(&self, device: &libusb::DeviceHandle) {
        if self.socket_id != 0 {
            return self.run_one(device, self.socket_id);
        }
        for i in 1..=4 {
            self.run_one(device, i);
        }
    }
}
impl ConfigStatus {
    fn run_one(&self, device: &libusb::DeviceHandle, id: u8) {
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

fn parse_cmd() -> Result<Box<dyn ConfigCMD>, &'static str> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Missing cmd");
    }
    match args[1].as_str() {
        "start" => ConfigStart::parse(&args[2..]),
        "stop" => ConfigStop::parse(&args[2..]),
        "status" => ConfigStatus::parse(&args[2..]),
        _ => Err("Unknwon option"),
    }
}

fn usage() -> ! {
    let args: Vec<String> = env::args().collect();
    println!("Usage: ");
    println!("{} status [socket_id]", args[0]);
    println!("{} start   socket_id", args[0]);
    println!("{} stop    socket_id", args[0]);

    process::exit(1);
}

fn main() {
    let cmd = parse_cmd().unwrap_or_else(|err| {
        println!("Error {}", err);
        println!("");
        usage();
    });

    let context = libusb::Context::new().unwrap();
    let device = context.open_device_with_vid_pid(0x04b4, 0xfd15).unwrap();

    cmd.run(&device);
}
