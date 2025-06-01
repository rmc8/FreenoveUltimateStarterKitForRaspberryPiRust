use rppal::gpio::{Gpio, Level, Trigger};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const LED_PIN: u8 = 17;
const BTN_PIN: u8 = 18;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Program is starting...");
    let gpio = Gpio::new()?;
    let mut led_pin = gpio.get(LED_PIN)?.into_output();
    let mut btn_pin = gpio.get(BTN_PIN)?.into_input();
    led_pin.set_low();
    // ボタンの割り込み設定
    btn_pin.set_interrupt(Trigger::FallingEdge, None);
    // Ctrl+Cが押されたら終了
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;
    println!("Waiting for button press...");
    while running.load(Ordering::SeqCst) {
        if let Some(_) = btn_pin.poll_interrupt(true, Some(Duration::from_millis(1)))? {
            // LEDの状態をトグル
            if led_pin.is_set_low() {
                led_pin.set_high();
                println!("Led turned on >>>");
            } else {
                led_pin.set_low();
                println!("Led turned off <<<");
            }
        }
    }
    println!("Program is finished.");
    btn_pin.clear_interrupt();
    led_pin.set_low();
    Ok(())
}
