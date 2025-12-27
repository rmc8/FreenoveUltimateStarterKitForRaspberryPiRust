use std::error::Error;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rppal::gpio::Gpio;
use rppal::i2c::I2c;

const PCF8591_ADDR: u16 = 0x48;
const ADS7830_ADDR: u16 = 0x4b;

// GPIO Pins for RGB LED
const RED_PIN: u8 = 22;
const GREEN_PIN: u8 = 27;
const BLUE_PIN: u8 = 17;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Program is starting ...");

    // Initialize I2C - try multiple buses
    let buses = [1, 13, 14];
    let mut i2c_n = None;
    let mut is_pcf8591 = None;

    for &bus in &buses {
        println!("Checking I2C bus {} ...", bus);
        let mut i2c = match I2c::with_bus(bus) {
            Ok(i) => i,
            Err(_) => continue,
        };

        for _ in 0..3 {
            if i2c.set_slave_address(PCF8591_ADDR).is_ok() && i2c.read(&mut [0]).is_ok() {
                is_pcf8591 = Some(true);
                i2c_n = Some(i2c);
                break;
            } else if i2c.set_slave_address(ADS7830_ADDR).is_ok() && i2c.read(&mut [0]).is_ok() {
                is_pcf8591 = Some(false);
                i2c_n = Some(i2c);
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        if i2c_n.is_some() {
            println!("Found device on bus {}", bus);
            break;
        }
    }
    let (mut i2c, is_pcf8591) = match (i2c_n, is_pcf8591) {
        (Some(i), Some(p)) => (i, p),
        _ => {
            eprintln!("No correct I2C device (PCF8591 or ADS7830) found on buses [1, 13, 14].");
            eprintln!("Please check your wiring and ensure I2C is enabled.");
            eprintln!("Program Exit.");
            std::process::exit(-1);
        }
    };

    println!(
        "Detected I2C device: {}",
        if is_pcf8591 { "PCF8591" } else { "ADS7830" }
    );

    // Shared state for PWM
    let running = Arc::new(AtomicBool::new(true));
    let duty_r = Arc::new(AtomicU8::new(0));
    let duty_g = Arc::new(AtomicU8::new(0));
    let duty_b = Arc::new(AtomicU8::new(0));

    // Spawn PWM thread
    let pwm_handle = {
        let running = running.clone();
        let duty_r = duty_r.clone();
        let duty_g = duty_g.clone();
        let duty_b = duty_b.clone();

        thread::spawn(move || {
            let gpio = match Gpio::new() {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("Failed to access GPIO: {}", e);
                    return;
                }
            };

            let mut pin_r = gpio.get(RED_PIN).unwrap().into_output();
            let mut pin_g = gpio.get(GREEN_PIN).unwrap().into_output();
            let mut pin_b = gpio.get(BLUE_PIN).unwrap().into_output();

            // 1 kHz frequency = 1000 us period
            let period_micros = 1000u64;

            while running.load(Ordering::SeqCst) {
                let dr = duty_r.load(Ordering::SeqCst) as u64;
                let dg = duty_g.load(Ordering::SeqCst) as u64;
                let db = duty_b.load(Ordering::SeqCst) as u64;

                // Simple Software PWM for 3 channels
                // We use 100 steps for granularity to keep CPU usage reasonable
                // Alternatively, we could use rppal's hardware PWM if available or its own SoftPwm
                // But for consistency with Softlight example, we'll do a simple bit-banging approach or use rppal's SoftPwm.
                // Actually, rppal's OutputPin has set_pwm which is easier.

                // Using rppal's built-in software PWM for simplicity and efficiency
                let _ = pin_r.set_pwm(
                    Duration::from_micros(period_micros),
                    Duration::from_micros(period_micros * dr / 255),
                );
                let _ = pin_g.set_pwm(
                    Duration::from_micros(period_micros),
                    Duration::from_micros(period_micros * dg / 255),
                );
                let _ = pin_b.set_pwm(
                    Duration::from_micros(period_micros),
                    Duration::from_micros(period_micros * db / 255),
                );

                thread::sleep(Duration::from_millis(10));
            }
            // Turn off LEDs on exit
            pin_r.set_low();
            pin_g.set_low();
            pin_b.set_low();
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
        let mut read_adc = |channel: u8| -> Result<u8, Box<dyn Error>> {
            if is_pcf8591 {
                i2c.set_slave_address(PCF8591_ADDR)?;
                i2c.write(&[0x40 | channel])?;
                let mut buf = [0u8; 1];
                i2c.read(&mut buf)?; // Dummy read
                i2c.read(&mut buf)?; // Actual read
                Ok(buf[0])
            } else {
                // ADS7830
                // Command byte: 1 (SD) | Channel (3 bits) | 01 (Internal Ref) | 00 (Unused)
                // Channel 0: 0x84, Channel 1: 0xc4, Channel 2: 0x94, Channel 3: 0xd4...
                // Actually, ADS7830 channel mapping:
                // Ch0: 0x84, Ch1: 0xC4, Ch2: 0x94, Ch3: 0xD4, Ch4: 0xA4, Ch5: 0xE4, Ch6: 0xB4, Ch7: 0xF4
                let cmd = match channel {
                    0 => 0x84,
                    1 => 0xc4,
                    2 => 0x94,
                    3 => 0xd4,
                    4 => 0xa4,
                    5 => 0xe4,
                    6 => 0xb4,
                    7 => 0xf4,
                    _ => 0x84,
                };
                i2c.set_slave_address(ADS7830_ADDR)?;
                i2c.write(&[cmd])?;
                let mut buf = [0u8; 1];
                i2c.read(&mut buf)?;
                Ok(buf[0])
            }
        };

        let val_r = read_adc(0).unwrap_or(0);
        let val_g = read_adc(1).unwrap_or(0);
        let val_b = read_adc(2).unwrap_or(0);

        duty_r.store(val_r, Ordering::SeqCst);
        duty_g.store(val_g, Ordering::SeqCst);
        duty_b.store(val_b, Ordering::SeqCst);

        println!(
            "ADC Value val_Red: {}, val_Green: {}, val_Blue: {}",
            val_r, val_g, val_b
        );

        thread::sleep(Duration::from_millis(10));
    }

    let _ = pwm_handle.join();
    Ok(())
}
