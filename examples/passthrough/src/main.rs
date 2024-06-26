#![no_std]
#![no_main]

use daisy_embassy::{
    audio::{Fs, InterleavedBlock, Start, HALF_DMA_BUFFER_LENGTH},
    board::DaisyPeripherals,
    embassy_sync::{blocking_mutex::raw::NoopRawMutex, zerocopy_channel::Channel},
    hal::{self, time::Hertz},
    pins::{DaisyPins, USB2Pins, WM8731Pins},
    DaisyBoard,
};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
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
        //default as PLL1_Q?
        //use hal::pac::rcc::vals::Saisel;
        //config.rcc.mux.sai1sel = Saisel::PLL1_Q;
    }

    let p = hal::init(config);
    let daisy_p = DaisyPeripherals {
        daisy_pins: DaisyPins {
            SEED_PIN_0: p.PB12,
            SEED_PIN_1: p.PC11,
            SEED_PIN_2: p.PC10,
            SEED_PIN_3: p.PC9,
            SEED_PIN_4: p.PC8,
            SEED_PIN_5: p.PD2,
            SEED_PIN_6: p.PC12,
            SEED_PIN_7: p.PG10,
            SEED_PIN_8: p.PG11,
            SEED_PIN_9: p.PB4,
            SEED_PIN_10: p.PB5,
            SEED_PIN_11: p.PB8,
            SEED_PIN_12: p.PB9,
            SEED_PIN_13: p.PB6,
            SEED_PIN_14: p.PB7,
            SEED_PIN_15: p.PC0,
            SEED_PIN_16: p.PA3,
            SEED_PIN_17: p.PB1,
            SEED_PIN_18: p.PA7,
            SEED_PIN_19: p.PA6,
            SEED_PIN_20: p.PC1,
            SEED_PIN_21: p.PC4,
            SEED_PIN_22: p.PA5,
            SEED_PIN_23: p.PA4,
            SEED_PIN_24: p.PA1,
            SEED_PIN_25: p.PA0,
            SEED_PIN_26: p.PD11,
            SEED_PIN_27: p.PG9,
            SEED_PIN_28: p.PA2,
            SEED_PIN_29: p.PB14,
            SEED_PIN_30: p.PB15,
        },
        led_user_pin: p.PC7,
        wm8731_pin: WM8731Pins {
            SCL: p.PH4,
            SDA: p.PB11,
            MCLK_A: p.PE2,
            SCK_A: p.PE5,
            FS_A: p.PE4,
            SD_A: p.PE6,
            SD_B: p.PE3,
        },
        audio_peripherals: daisy_embassy::audio::Peripherals {
            sai1: p.SAI1,
            i2c2: p.I2C2,
            dma1_ch1: p.DMA1_CH1,
            dma1_ch2: p.DMA1_CH2,
        },
        usb2_pins: USB2Pins {
            DN: p.PA11,
            DP: p.PA12,
        },
        usb_otg_fs: p.USB_OTG_FS,
    };
    let board = DaisyBoard::new(daisy_p, Fs::Fs48000, Fs::Fs48000);
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
