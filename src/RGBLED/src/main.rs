use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use rppal::gpio::{Gpio, OutputPin};
use rand::Rng;

// Pythonスクリプトに合わせたGPIOピン設定 (BCM番号)
const RED_PIN: u8 = 17;
const GREEN_PIN: u8 = 18;
const BLUE_PIN: u8 = 27;

/// ソフトウェアPWMを管理するスレッドを起動する関数
///
/// # Arguments
/// * `pin_num` - 制御するGPIOピン番号
/// * `duty_cycle` - 共有されるデューティサイクル (0.0から1.0)
/// * `running` - プログラムの実行状態を管理するフラグ
///
/// # Returns
/// * `Result<JoinHandle<()>, Box<dyn Error>>` - スレッドのJoinHandle
fn run_pwm_thread(
    pin_num: u8,
    duty_cycle: Arc<Mutex<f64>>,
    running: Arc<AtomicBool>,
) -> Result<JoinHandle<()>, Box<dyn Error>> {
    let gpio = Gpio::new()?;
    let mut pin = gpio.get(pin_num)?.into_output();

    let handle = thread::spawn(move || {
        // 100Hz相当の周期 (10,000マイクロ秒)
        let period = Duration::from_micros(10000); 
        
        while running.load(Ordering::SeqCst) {
            let current_duty_cycle = *duty_cycle.lock().unwrap();

            // デューティサイクルに基づいてオン/オフ時間を計算
            // Common-Anode LEDの場合、LOWで点灯、HIGHで消灯
            let on_time = period.mul_f64(current_duty_cycle);
            let off_time = period.saturating_sub(on_time);

            if !on_time.is_zero() {
                pin.set_low(); // 点灯
                thread::sleep(on_time);
            }
            if !off_time.is_zero() {
                pin.set_high(); // 消灯
                thread::sleep(off_time);
            }
        }
        // 終了時にピンをリセット
        pin.set_high(); 
    });

    Ok(handle)
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Program is starting...");

    // Ctrl+Cでプログラムを終了するための設定
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    // 各色のデューティサイクルをスレッド間で共有するための変数
    // 初期値は0.0（消灯）に設定
    let r_duty = Arc::new(Mutex::new(0.0));
    let g_duty = Arc::new(Mutex::new(0.0));
    let b_duty = Arc::new(Mutex::new(0.0));

    // 各色を制御するPWMスレッドを起動
    let mut threads: Vec<JoinHandle<()>> = Vec::new();
    threads.push(run_pwm_thread(RED_PIN, r_duty.clone(), running.clone())?);
    threads.push(run_pwm_thread(GREEN_PIN, g_duty.clone(), running.clone())?);
    threads.push(run_pwm_thread(BLUE_PIN, b_duty.clone(), running.clone())?);

    let mut rng = rand::thread_rng();

    // メインループ：乱数を生成し、LEDの色を更新する
    while running.load(Ordering::SeqCst) {
        // 0から100の範囲でランダムな値を取得
        let r_val = rng.gen_range(0..=100);
        let g_val = rng.gen_range(0..=100);
        let b_val = rng.gen_range(0..=100);
        
        // 値をデューティサイクル (0.0〜1.0) に変換
        // PythonのgpiozeroのRGBLEDクラス(active_high=False)の動作に合わせる
        // 値が100のとき、デューティサイクルは1.0 (完全にオン) となる
        *r_duty.lock().unwrap() = r_val as f64 / 100.0;
        *g_duty.lock().unwrap() = g_val as f64 / 100.0;
        *b_duty.lock().unwrap() = b_val as f64 / 100.0;

        println!("r={}, g={}, b={}", r_val, g_val, b_val);

        // 1000ms待機
        thread::sleep(Duration::from_millis(1000));
    }
    
    println!("\nEnding program...");
    
    // すべてのスレッドが終了するのを待つ
    for handle in threads {
        handle.join().unwrap();
    }

    Ok(())
}
