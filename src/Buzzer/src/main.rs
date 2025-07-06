use rppal::gpio::{Gpio, InputPin, OutputPin, Trigger};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const BUZZER_PIN: u8 = 17;
const BTN_PIN: u8 = 18;
const POLL_TIMEOUT_MS: u64 = 10;

fn main() -> Result<(), Box<dyn Error>> {
    print_startup_message();
    
    let (mut buzzer_pin, mut btn_pin) = initialize_gpio()?;
    initialize_buzzer(&mut buzzer_pin);
    setup_button_interrupt(&mut btn_pin)?;
    
    let running = setup_signal_handler()?;
    
    println!("Waiting for button press...");
    
    run_interrupt_loop(&running, &mut buzzer_pin, &mut btn_pin)?;
    
    cleanup(&mut buzzer_pin, &mut btn_pin)?;
    
    Ok(())
}

fn print_startup_message() {
    println!("Program is starting...");
}

fn initialize_gpio() -> Result<(OutputPin, InputPin), Box<dyn Error>> {
    let gpio = Gpio::new()?;
    let buzzer_pin = gpio.get(BUZZER_PIN)?.into_output();
    let btn_pin = gpio.get(BTN_PIN)?.into_input();
    Ok((buzzer_pin, btn_pin))
}

fn initialize_buzzer(buzzer_pin: &mut OutputPin) {
    buzzer_pin.set_low();
}

fn setup_button_interrupt(btn_pin: &mut InputPin) -> Result<(), Box<dyn Error>> {
    btn_pin.set_interrupt(Trigger::Both, None)?;
    Ok(())
}

fn setup_signal_handler() -> Result<Arc<AtomicBool>, Box<dyn Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;
    Ok(running)
}

fn run_interrupt_loop(
    running: &Arc<AtomicBool>,
    buzzer_pin: &mut OutputPin,
    btn_pin: &mut InputPin,
) -> Result<(), Box<dyn Error>> {
    while running.load(Ordering::SeqCst) {
        if let Some(_) = btn_pin.poll_interrupt(true, Some(Duration::from_millis(POLL_TIMEOUT_MS)))? {
            handle_button_interrupt(buzzer_pin, btn_pin);
        }
    }
    Ok(())
}

fn handle_button_interrupt(buzzer_pin: &mut OutputPin, btn_pin: &InputPin) {
    if is_button_pressed(btn_pin) {
        turn_on_buzzer(buzzer_pin);
        print_buzzer_on_message();
    } else {
        turn_off_buzzer(buzzer_pin);
        print_buzzer_off_message();
    }
}

fn is_button_pressed(btn_pin: &InputPin) -> bool {
    btn_pin.is_low()
}

fn turn_on_buzzer(buzzer_pin: &mut OutputPin) {
    buzzer_pin.set_high();
}

fn turn_off_buzzer(buzzer_pin: &mut OutputPin) {
    buzzer_pin.set_low();
}

fn print_buzzer_on_message() {
    println!("Button is pressed, buzzer turned on >>>");
}

fn print_buzzer_off_message() {
    println!("Button is released, buzzer turned off <<<");
}

fn cleanup(buzzer_pin: &mut OutputPin, btn_pin: &mut InputPin) -> Result<(), Box<dyn Error>> {
    println!("Ending program");
    let _ = btn_pin.clear_interrupt();
    turn_off_buzzer(buzzer_pin);
    Ok(())
}
