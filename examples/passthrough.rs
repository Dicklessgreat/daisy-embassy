#![no_std]
#![no_main]

use daisy_embassy::{audio::HALF_DMA_BUFFER_LENGTH, hal, new_daisy_board};
use defmt::debug;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    debug!("====program start====");
    let config = daisy_embassy::default_rcc();
    let p = hal::init(config);
    let board = new_daisy_board!(p);
    let (mut interface, (mut to_interface, mut from_interface)) = board
        .audio_peripherals
        .prepare_interface(Default::default())
        .await;

    let audio_callback_fut = async {
        let mut buf = [0; HALF_DMA_BUFFER_LENGTH];
        loop {
            let rx = from_interface.receive().await;
            buf.copy_from_slice(rx);
            from_interface.receive_done();

            let tx = to_interface.send().await;
            tx.copy_from_slice(&buf);
            to_interface.send_done();
        }
    };
    join(interface.start(), audio_callback_fut).await;
}
