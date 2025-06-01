use rppal::gpio::{Gpio, Level};
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

const LED_PIN: u8 = 17;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Program is starting...");
    let gpio = Gpio::new()?;
    let mut pin = gpio.get(LED_PIN)?.into_output();
    println!("LED will blink every 1 second.");

    loop {
        pin.write(Level::High);
        println!("LED turned on >>>");
        sleep(Duration::from_secs(1));
        pin.write(Level::Low);
        println!("LED turned off <<<");
        sleep(Duration::from_secs(1));
    }
}
