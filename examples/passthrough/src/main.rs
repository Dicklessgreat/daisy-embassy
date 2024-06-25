#![no_std]
#![no_main]

use daisy_embassy::{
    audio::{Fs, InterleavedBlock, Start, HALF_DMA_BUFFER_LENGTH},
    embassy_sync::{blocking_mutex::raw::NoopRawMutex, zerocopy_channel::Channel},
    hal::{self, time::Hertz},
    DaisyBoard,
};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = hal::Config::default();
    {
        use hal::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8),
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(16),
            mode: HseMode::Oscillator,
        })
        //default as PLL1_Q?
        //use hal::pac::rcc::vals::Saisel;
        //config.rcc.mux.sai1sel = Saisel::PLL1_Q;
    }
    let board = DaisyBoard::new(config, Fs::Fs48000, Fs::Fs48000);
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
