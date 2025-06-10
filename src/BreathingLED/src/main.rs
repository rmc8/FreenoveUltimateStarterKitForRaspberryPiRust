use rppal::gpio::{Gpio, OutputPin};
use rppal::pwm::{Channel, Polarity, Pwm};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const LED_PIN: u8 = 18;
const PWM_FREQUENCY: f64 = 1000.0;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Breathing LED...");
    println!("Press Ctrl+C to quit");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\nExiting...");
        r.store(false, Ordering::SeqCst);
    })?;

    let gpio = Gpio::new()?;
    let mut led = gpio.get(LED_PIN)?.into_output();

    println!("Starting software PWM on GPIO pin {}", LED_PIN);

    let mut brightness = 0.0;
    let mut increasing = true;
    let step = 0.01;
    let delay = Duration::from_millis(10);

    while running.load(Ordering::SeqCst) {
        led.set_pwm_frequency(PWM_FREQUENCY, brightness)?;

        if increasing {
            brightness += step;
            if brightness >= 1.0 {
                brightness = 1.0;
                increasing = false;
            }
        } else {
            brightness -= step;
            if brightness <= 0.0 {
                brightness = 0.0;
                increasing = true;
            }
        }

        thread::sleep(delay);
    }

    led.clear_pwm()?;
    led.set_low();
    println!("Breathing LED stopped");

    Ok(())
}
