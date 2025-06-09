use rppal::gpio::{Gpio, Level};
use std::error::Error;

const LED_PIN: u8 = 17;
const BTN_PIN: u8 = 18;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Program is starting...");
    let gpio = Gpio::new()?;
    let mut led_pin = gpio.get(LED_PIN)?.into_output();
    let btn_pin = gpio.get(BTN_PIN)?.into_input();
    loop {
        if btn_pin.is_low() {
            // led_pin.set_high();
            led_pin.write(Level::High);
            println!("Button is pressed, led turned on >>>");
        } else {
            // led_pin.set_low();
            led_pin.write(Level::Low);
            println!("Button is released, led turned off <<<");
        }
    }
}
