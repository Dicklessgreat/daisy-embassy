#![no_std]
#![no_main]

use daisy_embassy::pins::{DaisyPins, USB2Pins, WM8731Pins};
use daisy_embassy::{new_daisy_p, DaisyBoard};
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");
    let daisy_p = DaisyBoard::new(new_daisy_p!(p));
    let mut led = daisy_p.user_led;

    loop {
        info!("on");
        led.on();
        Timer::after_millis(300).await;

        info!("off");
        led.off();
        Timer::after_millis(300).await;
    }
}
