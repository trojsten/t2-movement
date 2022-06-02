use anyhow::Result;
use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread::sleep;
use std::time::Duration;

use rppal::gpio::{Gpio, IoPin, Mode};

struct Movement {
    pin_up: IoPin,
    pin_down: IoPin,
    pin_stop: IoPin,
}

fn make_pin(gpio: &Gpio, pin: u8) -> Result<IoPin> {
    let pin = gpio.get(pin)?;
    let mut pin = pin.into_io(Mode::Input);
    pin.set_pullupdown(rppal::gpio::PullUpDown::Off);
    Ok(pin)
}

impl Movement {
    fn new(up: u8, down: u8, stop: u8) -> Result<Self> {
        let gpio = Gpio::new()?;
        Ok(Self {
            pin_up: make_pin(&gpio, up)?,
            pin_down: make_pin(&gpio, down)?,
            pin_stop: make_pin(&gpio, stop)?,
        })
    }

    fn get_pin(&mut self, cmd: Command) -> &mut IoPin {
        match cmd {
            Command::Up => &mut self.pin_up,
            Command::Down => &mut self.pin_down,
            Command::Stop => &mut self.pin_stop,
        }
    }

    pub fn perform_command(&mut self, cmd: Command) -> Result<()> {
        eprintln!("Performing command: {:?}", cmd);
        let pin = self.get_pin(cmd);

        pin.set_pullupdown(rppal::gpio::PullUpDown::PullUp);
        sleep(Duration::from_millis(250));
        pin.set_pullupdown(rppal::gpio::PullUpDown::Off);
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
