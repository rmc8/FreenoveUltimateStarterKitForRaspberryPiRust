use std::error::Error;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rppal::gpio::Gpio;
use rppal::i2c::I2c;

const PCF8591_ADDR: u16 = 0x48;
const ADS7830_ADDR: u16 = 0x4b;
// GPIO 17 (BCM)
const LED_PIN: u8 = 17;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Program is starting ...");

    // Initialize I2C
    let mut i2c = I2c::new()?;

    // Detect I2C device with retries
    let mut is_pcf8591 = None;

    for _ in 0..5 {
        if i2c.set_slave_address(PCF8591_ADDR).is_ok() && i2c.read(&mut [0]).is_ok() {
            is_pcf8591 = Some(true);
            break;
        } else if i2c.set_slave_address(ADS7830_ADDR).is_ok() && i2c.read(&mut [0]).is_ok() {
            is_pcf8591 = Some(false);
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    let is_pcf8591 = match is_pcf8591 {
        Some(v) => v,
        None => {
            eprintln!("No correct I2C address found after retries,");
            eprintln!("Please use command 'i2cdetect -y 1' to check the I2C address!");
            eprintln!("Program Exit.");
            std::process::exit(-1);
        }
    };

    println!(
        "Detected I2C device: {}",
        if is_pcf8591 { "PCF8591" } else { "ADS7830" }
    );

    // Shared state for SoftPWM
    let running = Arc::new(AtomicBool::new(true));
    let duty_cycle = Arc::new(AtomicU8::new(0));

    // Spawn SoftPWM thread
    let pwm_handle = {
        let running = running.clone();
        let duty_cycle = duty_cycle.clone();
        thread::spawn(move || {
            let gpio = match Gpio::new() {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("Failed to access GPIO: {}", e);
                    return;
                }
            };

            let mut pin = match gpio.get(LED_PIN) {
                Ok(p) => p.into_output(),
                Err(e) => {
                    eprintln!("Failed to get GPIO pin {}: {}", LED_PIN, e);
                    return;
                }
            };

            // 1 kHz frequency = 1000 us period
            let period_micros = 1000u64;

            while running.load(Ordering::SeqCst) {
                let duty = duty_cycle.load(Ordering::SeqCst) as u64;

                if duty == 0 {
                    pin.set_low();
                    thread::sleep(Duration::from_micros(period_micros));
                } else if duty == 255 {
                    pin.set_high();
                    thread::sleep(Duration::from_micros(period_micros));
                } else {
                    // Calculate on/off times
                    // duty is 0..255
                    let on_time = (period_micros * duty) / 255;
                    let off_time = period_micros - on_time;

                    pin.set_high();
                    thread::sleep(Duration::from_micros(on_time));
                    if off_time > 0 {
                        pin.set_low();
                        thread::sleep(Duration::from_micros(off_time));
                    }
                }
            }
            // Turn off LED on exit
            pin.set_low();
        })
    };

    // Setup CTRL-C handler
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        println!("\nEnding program");
        running_clone.store(false, Ordering::SeqCst);
    })?;

    // Main loop
    while running.load(Ordering::SeqCst) {
        let value_result: Result<u8, Box<dyn Error>> = if is_pcf8591 {
            // PCF8591
            i2c.set_slave_address(PCF8591_ADDR)
                .and_then(|_| i2c.write(&[0x40]))
                .and_then(|_| {
                    let mut buf = [0u8; 1];
                    i2c.read(&mut buf)?;
                    i2c.read(&mut buf)?;
                    Ok(buf[0])
                })
                .map_err(|e| e.into()) // Convert rppal::i2c::Error to Box<dyn Error>
        } else {
            // ADS7830
            i2c.set_slave_address(ADS7830_ADDR)
                .and_then(|_| i2c.write(&[0x84]))
                .and_then(|_| {
                    let mut buf = [0u8; 1];
                    i2c.read(&mut buf)?;
                    Ok(buf[0])
                })
                .map_err(|e| e.into()) // Convert rppal::i2c::Error to Box<dyn Error>
        };

        match value_result {
            Ok(value) => {
                // Update PWM duty cycle
                duty_cycle.store(value, Ordering::SeqCst);

                // Display info
                // Voltage reference 3.3V
                let voltage = (value as f64 / 255.0) * 3.3;
                println!("ADC Value : {}, Voltage : {:.2}", value, voltage);
            }
            Err(e) => {
                eprintln!("Error reading I2C: {}", e);
                // Optional: add a small delay or just continue to retry
            }
        }

        thread::sleep(Duration::from_millis(30));
    }

    // Wait for PWM thread to finish
    let _ = pwm_handle.join();

    Ok(())
}
