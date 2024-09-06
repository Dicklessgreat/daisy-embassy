#![no_std]
#![no_main]

use daisy_embassy::{hal, new_daisy_board, audio::HALF_DMA_BUFFER_LENGTH};
use defmt::debug;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    debug!("====program start====");
    let config = daisy_embassy::default_rcc();
    let p = hal::init(config);
    let board = new_daisy_board!(p);
    let mut interface = board
        .audio_peripherals
        .prepare_interface(Default::default())
        .await;

    interface
        .start(|input, output| {
            output.copy_from_slice(input);
        })
        .await;
}
