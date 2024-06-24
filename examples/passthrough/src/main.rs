#![no_std]
#![no_main]

use daisy_embassy::{
    audio::{Fs, InterleavedBlock, Start, HALF_DMA_BUFFER_LENGTH},
    embassy_sync::{blocking_mutex::raw::NoopRawMutex, zerocopy_channel::Channel},
    DaisyBoard,
};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let board = DaisyBoard::new(Default::default(), Fs::Fs48000, Fs::Fs48000);
    let mut interface = board.interface;

    static TO_INTERFACE_BUF: StaticCell<[InterleavedBlock; 2]> = StaticCell::new();
    let to_interface_buf = TO_INTERFACE_BUF.init([[0; HALF_DMA_BUFFER_LENGTH]; 2]);
    static TO_INTERFACE: StaticCell<Channel<'_, NoopRawMutex, InterleavedBlock>> =
        StaticCell::new();
    let (mut to_interface, client_to_if) =
        TO_INTERFACE.init(Channel::new(to_interface_buf)).split();
    static FROM_INTERFACE_BUF: StaticCell<[InterleavedBlock; 2]> = StaticCell::new();
    let from_interface_buf = FROM_INTERFACE_BUF.init([[0; HALF_DMA_BUFFER_LENGTH]; 2]);
    static FROM_INTERFACE: StaticCell<Channel<'_, NoopRawMutex, InterleavedBlock>> =
        StaticCell::new();
    let (if_to_client, mut from_interface) = FROM_INTERFACE
        .init(Channel::new(from_interface_buf))
        .split();
    let interface_fut = async {
        interface
            .start(Start {
                client_to_if,
                if_to_client,
            })
            .await
    };

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
    join(interface_fut, audio_callback_fut).await;
}
