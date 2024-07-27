#![no_std]
pub mod audio;
pub mod board;
pub mod flash;
pub mod led;
pub mod pins;
pub mod sdram;
pub mod usb;

pub use board::DaisyBoard;
pub use embassy_stm32 as hal;

pub fn default_rcc() -> hal::Config {
    let mut config = hal::Config::default();
    use hal::rcc::*;
    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL240,
        divp: Some(PllDiv::DIV2),
        divq: Some(PllDiv::DIV20),
        divr: Some(PllDiv::DIV2),
    });
    config.rcc.pll2 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL50,
        divp: None,
        divq: None,
        divr: Some(PllDiv::DIV2),
    });
    config.rcc.pll3 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV6,
        mul: PllMul::MUL295,
        divp: Some(PllDiv::DIV16),
        divq: Some(PllDiv::DIV4),
        divr: Some(PllDiv::DIV32),
    });
    config.rcc.sys = Sysclk::PLL1_P; // 480MHz
    config.rcc.mux.fmcsel = hal::pac::rcc::vals::Fmcsel::PLL2_R; // 100MHz
    config.rcc.mux.sai1sel = hal::pac::rcc::vals::Saisel::PLL3_P; // 49.2MHz
    config.rcc.mux.usbsel = hal::pac::rcc::vals::Usbsel::PLL1_Q; // 48MHz
    config.rcc.ahb_pre = AHBPrescaler::DIV2; // 240 MHz
    config.rcc.apb1_pre = APBPrescaler::DIV2; // 120 MHz
    config.rcc.apb2_pre = APBPrescaler::DIV2; // 120 MHz
    config.rcc.apb3_pre = APBPrescaler::DIV2; // 120 MHz
    config.rcc.apb4_pre = APBPrescaler::DIV2; // 120 MHz
    config.rcc.voltage_scale = VoltageScale::Scale0;
    config.rcc.hse = Some(Hse {
        freq: hal::time::Hertz::mhz(16),
        mode: HseMode::Oscillator,
    });
    config
}

#[macro_export]
macro_rules! new_daisy_board {
    ($p:ident) => {
        daisy_embassy::board::DaisyBoard {
            pins: daisy_embassy::pins::DaisyPins {
                d0: $p.PB12,
                d1: $p.PC11,
                d2: $p.PC10,
                d3: $p.PC9,
                d4: $p.PC8,
                d5: $p.PD2,
                d6: $p.PC12,
                d7: $p.PG10,
                d8: $p.PG11,
                d9: $p.PB4,
                d10: $p.PB5,
                d11: $p.PB8,
                d12: $p.PB9,
                d13: $p.PB6,
                d14: $p.PB7,
                d15: $p.PC0,
                d16: $p.PA3,
                d17: $p.PB1,
                d18: $p.PA7,
                d19: $p.PA6,
                d20: $p.PC1,
                d21: $p.PC4,
                d22: $p.PA5,
                d23: $p.PA4,
                d24: $p.PA1,
                d25: $p.PA0,
                d26: $p.PD11,
                d27: $p.PG9,
                d28: $p.PA2,
                d29: $p.PB14,
                d30: $p.PB15,
            },
            user_led: daisy_embassy::led::UserLed::new($p.PC7),

            audio_peripherals: daisy_embassy::audio::AudioPeripherals {
                wm8731: daisy_embassy::pins::WM8731Pins {
                    SCL: $p.PH4,
                    SDA: $p.PB11,
                    MCLK_A: $p.PE2,
                    SCK_A: $p.PE5,
                    FS_A: $p.PE4,
                    SD_A: $p.PE6,
                    SD_B: $p.PE3,
                },
                sai1: $p.SAI1,
                i2c2: $p.I2C2,
                dma1_ch1: $p.DMA1_CH1,
                dma1_ch2: $p.DMA1_CH2,
            },
            flash: daisy_embassy::flash::FlashBuilder {
                pins: daisy_embassy::pins::FlashPins {
                    IO0: $p.PF8,
                    IO1: $p.PF9,
                    IO2: $p.PF7,
                    IO3: $p.PF6,
                    SCK: $p.PF10,
                    CS: $p.PG6,
                },
                qspi: $p.QUADSPI,
            },
            sdram: daisy_embassy::sdram::SdRamBuilder {
                pins: daisy_embassy::pins::SdRamPins {
                    dd0: $p.PD0,
                    dd1: $p.PD1,
                    dd8: $p.PD8,
                    dd9: $p.PD9,
                    dd10: $p.PD10,
                    dd14: $p.PD14,
                    dd15: $p.PD15,
                    ee0: $p.PE0,
                    ee1: $p.PE1,
                    ee7: $p.PE7,
                    ee8: $p.PE8,
                    ee9: $p.PE9,
                    ee10: $p.PE10,
                    ee11: $p.PE11,
                    ee12: $p.PE12,
                    ee13: $p.PE13,
                    ee14: $p.PE14,
                    ee15: $p.PE15,
                    ff0: $p.PF0,
                    ff1: $p.PF1,
                    ff2: $p.PF2,
                    ff3: $p.PF3,
                    ff4: $p.PF4,
                    ff5: $p.PF5,
                    ff11: $p.PF11,
                    ff12: $p.PF12,
                    ff13: $p.PF13,
                    ff14: $p.PF14,
                    ff15: $p.PF15,
                    gg0: $p.PG0,
                    gg1: $p.PG1,
                    gg2: $p.PG2,
                    gg4: $p.PG4,
                    gg5: $p.PG5,
                    gg8: $p.PG8,
                    gg15: $p.PG15,
                    hh2: $p.PH2,
                    hh3: $p.PH3,
                    hh5: $p.PH5,
                    hh8: $p.PH8,
                    hh9: $p.PH9,
                    hh10: $p.PH10,
                    hh11: $p.PH11,
                    hh12: $p.PH12,
                    hh13: $p.PH13,
                    hh14: $p.PH14,
                    hh15: $p.PH15,
                    ii0: $p.PI0,
                    ii1: $p.PI1,
                    ii2: $p.PI2,
                    ii3: $p.PI3,
                    ii4: $p.PI4,
                    ii5: $p.PI5,
                    ii6: $p.PI6,
                    ii7: $p.PI7,
                    ii9: $p.PI9,
                    ii10: $p.PI10,
                },
                instance: $p.FMC,
            },
            usb_peripherals: daisy_embassy::usb::UsbPeripherals {
                pins: daisy_embassy::pins::USB2Pins {
                    DN: $p.PA11,
                    DP: $p.PA12,
                },
                usb_otg_fs: $p.USB_OTG_FS,
            },
            boot: $p.PG3,
        }
    };
}
