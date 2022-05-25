use anyhow::{Context, Result};
use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread::sleep;
use std::time::Duration;

use sysfs_gpio::{Direction, Pin};

struct Movement {
    pin_up: Pin,
    pin_down: Pin,
    pin_stop: Pin,
}

fn make_pin(pin: u64) -> Result<Pin> {
    let p = Pin::new(pin);
    p.export()?;
    p.set_direction(Direction::In)
        .context("set pin direction")?;
    Ok(p)
}

impl Movement {
    fn new(up: u64, down: u64, stop: u64) -> Result<Self> {
        Ok(Self {
            pin_up: make_pin(up)?,
            pin_down: make_pin(down)?,
            pin_stop: make_pin(stop)?,
        })
    }

    fn get_pin(&self, cmd: Command) -> Pin {
        match cmd {
            Command::Up => self.pin_up,
            Command::Down => self.pin_down,
            Command::Stop => self.pin_stop,
        }
    }

    pub fn perform_command(&mut self, cmd: Command) -> Result<()> {
        eprintln!("Performing command: {:?}", cmd);
        let pin = self.get_pin(cmd);
        pin.set_direction(Direction::High)?;
        // pin.set_value(1)?;
        sleep(Duration::from_millis(1500));
        pin.set_direction(Direction::In)?;
        eprintln!("Done");
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Command {
    Up,
    Down,
    Stop,
}

fn process_client(mut stream: TcpStream, movement: &mut Movement) -> Result<()> {
    let mut buf = [0u8; 1];
    stream.read(&mut buf)?;

    let command = match buf[0] {
        1 | b'u' => Command::Up,
        2 | b'd' => Command::Down,
        3 | b's' => Command::Stop,
        _ => {
            stream.write(&[1u8])?;
            return Ok(());
        }
    };
    stream.shutdown(Shutdown::Both)?;

    movement.perform_command(command)?;

    Ok(())
}

fn main() -> Result<()> {
    let mut movement = Movement::new(23, 24, 25)?;
    let listener = TcpListener::bind("0.0.0.0:1113")?;
    for stream in listener.incoming() {
        match process_client(stream?, &mut movement) {
            Ok(_) => {}
            Err(e) => eprintln!("error handling client: {}", e),
        };
    }
    Ok(())
}
