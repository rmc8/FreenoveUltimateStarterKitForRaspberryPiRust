use rppal::gpio::{Gpio, InputPin, OutputPin};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const BUZZER_PIN: u8 = 17;
const BTN_PIN: u8 = 18;
const ALERTOR_FREQUENCY: f64 = 220.0; // 220Hz
const ALERTOR_DUTY_CYCLE: f64 = 0.5;  // 50% duty cycle
const LOOP_DELAY_MS: u64 = 10;
const ALERTOR_DURATION_MS: u64 = 100;

fn main() -> Result<(), Box<dyn Error>> {
    print_startup_message();
    
    let (mut buzzer_pin, btn_pin) = initialize_gpio()?;
    initialize_buzzer(&mut buzzer_pin);
    
    let running = setup_signal_handler()?;
    
    println!("Waiting for button press...");
    
    run_main_loop(&running, &mut buzzer_pin, &btn_pin)?;
    
    cleanup(&mut buzzer_pin)?;
    
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

fn setup_signal_handler() -> Result<Arc<AtomicBool>, Box<dyn Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;
    Ok(running)
}

fn run_main_loop(
    running: &Arc<AtomicBool>,
    buzzer_pin: &mut OutputPin,
    btn_pin: &InputPin,
) -> Result<(), Box<dyn Error>> {
    while running.load(Ordering::SeqCst) {
        if is_button_pressed(btn_pin) {
            play_alertor_sound(buzzer_pin)?;
            print_alertor_on_message();
        } else {
            stop_alertor_sound(buzzer_pin)?;
            print_alertor_off_message();
        }
        
        thread::sleep(Duration::from_millis(LOOP_DELAY_MS));
    }
    Ok(())
}

fn is_button_pressed(btn_pin: &InputPin) -> bool {
    btn_pin.is_low()
}

fn play_alertor_sound(buzzer_pin: &mut OutputPin) -> Result<(), Box<dyn Error>> {
    buzzer_pin.set_pwm_frequency(ALERTOR_FREQUENCY, ALERTOR_DUTY_CYCLE)?;
    thread::sleep(Duration::from_millis(ALERTOR_DURATION_MS));
    Ok(())
}

fn stop_alertor_sound(buzzer_pin: &mut OutputPin) -> Result<(), Box<dyn Error>> {
    buzzer_pin.clear_pwm()?;
    buzzer_pin.set_low();
    Ok(())
}

fn print_alertor_on_message() {
    println!("alertor turned on >>> ");
}

fn print_alertor_off_message() {
    println!("alertor turned off <<<");
}

fn cleanup(buzzer_pin: &mut OutputPin) -> Result<(), Box<dyn Error>> {
    println!("Ending program");
    stop_alertor_sound(buzzer_pin)?;
    Ok(())
}
