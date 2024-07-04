#![no_std]
#![no_main]

use daisy_embassy::{
    audio::HALF_DMA_BUFFER_LENGTH,
    hal::{self, time::Hertz},
    new_daisy_boad,
};
use defmt::debug;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    debug!("====program start====");
    let mut core = cortex_m::Peripherals::take().unwrap();
    core.SCB.enable_icache();
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
    }

    let p = hal::init(config);
    let board = new_daisy_boad!(p);
    let (mut interface, (mut to_interface, mut from_interface)) = board
        .audio_peripherals
        .prepare_interface(Default::default())
        .await;

    let interface_fut = interface.start();

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
