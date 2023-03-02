extern crate libusb;

use std::env;
use std::process;

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

        Ok(Config{ cmd: args[1].clone(), socket_id })
    }
}

fn usage() -> ! {
    let args: Vec<String> = env::args().collect();

    println!("Usage: ");
    println!("{} [status,start,stop] [socket_id]", args[0]);

    process::exit(1);
}

fn main() {
    let config = Config::build().unwrap_or_else(|err| {
        println!("Error {}", err);
        println!("");
        usage();
    });
}
