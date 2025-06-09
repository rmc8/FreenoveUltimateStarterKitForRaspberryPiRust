use rppal::gpio::{Gpio, Level};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    // Init
    println!("Program is starting...");
    let gpio = Gpio::new()?;
    const LED_PINS: [u8; 10] = [17, 18, 27, 22, 23, 24, 25, 2, 3, 8];
    let mut leds: Vec<_> = Vec::with_capacity(LED_PINS.len());
    for &pin_num in LED_PINS.iter() {
        let pin = gpio.get(pin_num)?.into_output();
        leds.push(pin);
    }

    // Ctrl+Cが押されたら終了
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    // Main loop
    while running.load(Ordering::SeqCst) {
        for i in 0..leds.len() {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            leds[i].write(Level::Low);
            sleep(Duration::from_millis(100));
            leds[i].write(Level::High);
        }
        for i in (0..leds.len()).rev() {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            leds[i].write(Level::Low);
            sleep(Duration::from_millis(100));
            leds[i].write(Level::High);
        }
    }
    // Cleanup
    for led in leds.iter_mut() {
        led.set_low();
    }
    Ok(())
}
