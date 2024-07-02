#![no_std]
pub mod audio;
pub mod board;
pub mod led;
pub mod pins;
pub mod usb;

pub use board::DaisyBoard;
pub use embassy_stm32 as hal;

#[macro_export]
macro_rules! new_daisy_boad {
    ($p:ident) => {
        daisy_embassy::board::DaisyBoard {
            pins: DaisyPins {
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
                wm8731: WM8731Pins {
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
            FMC: (),
            SDRAM: (),
            usb_peripherals: daisy_embassy::usb::UsbPeripherals {
                pins: USB2Pins {
                    DN: $p.PA11,
                    DP: $p.PA12,
                },
                usb_otg_fs: $p.USB_OTG_FS,
            },
            boot: $p.PG3,
        }
    };
}
