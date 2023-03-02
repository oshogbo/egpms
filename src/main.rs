extern crate libusb;

use std::env;
use std::process;
use std::time::Duration;

const TIMEOUT : Duration = Duration::from_secs(1);

struct Config {
    cmd: String,
    socket_id: u8,
}

impl Config {
    fn build() -> Result<Config, &'static str> {
        let args: Vec<String> = env::args().collect();

        if args.len() < 3 {
            return Err("not enough arguments");
        }
        let socket_id : u8 = match args[2].parse() {
            Ok(num) => num,
            Err(_) => return Err("Socket id has to be number"),
        };
        if socket_id < 1 || socket_id > 4 {
            return Err("Unknwon socket");
        }

        Ok(Config{ cmd: args[1].clone(), socket_id })
    }
}

fn usage() -> ! {
    let args: Vec<String> = env::args().collect();

    println!("Usage: ");
    println!("{} [status,start,stop] [socket_id]", args[0]);

    process::exit(1);
}

fn from_id_to_device(id: u8) -> u8 {
    0x03 * id
}

fn cmd_start(device: &libusb::DeviceHandle, socket_id: u8) {
    let devid = from_id_to_device(socket_id);
    let data = [devid, 0x01];
    let count = device.write_control(
        0x21,
        0x09,
        0x0300 + (devid as u16),
        0x0,
        &data,
        TIMEOUT,
    ).unwrap();
    println!("Done: {}", count);
}

fn cmd_stop(_device: &libusb::DeviceHandle, _socket_id: u8) {
    unimplemented!();
}

fn cmd_status(_device: &libusb::DeviceHandle, _socket_id: u8) {
    unimplemented!();
}

fn main() {
    let config = Config::build().unwrap_or_else(|err| {
        println!("Error {}", err);
        println!("");
        usage();
    });

    let context = libusb::Context::new().unwrap();
    let device = context.open_device_with_vid_pid(0x04b4, 0xfd15).unwrap();

    match config.cmd.as_str() {
        "start" => cmd_start(&device, config.socket_id),
        "stop" => cmd_stop(&device, config.socket_id),
        "status" => cmd_status(&device, config.socket_id),
        _ => {
            println!("Unknwon option");
            usage();
        },
    }
}
