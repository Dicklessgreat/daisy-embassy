#![no_std]
pub mod audio;
pub mod board;
pub mod led;
pub mod pins;
pub mod usb;

pub use audio::Fs;
pub use board::DaisyBoard;
pub use embassy_stm32 as hal;

#[macro_export]
macro_rules! new_daisy_p {
    ($p:ident) => {
        daisy_embassy::board::DaisyPeripherals {
            daisy_pins: DaisyPins {
                SEED_PIN_0: $p.PB12,
                SEED_PIN_1: $p.PC11,
                SEED_PIN_2: $p.PC10,
                SEED_PIN_3: $p.PC9,
                SEED_PIN_4: $p.PC8,
                SEED_PIN_5: $p.PD2,
                SEED_PIN_6: $p.PC12,
                SEED_PIN_7: $p.PG10,
                SEED_PIN_8: $p.PG11,
                SEED_PIN_9: $p.PB4,
                SEED_PIN_10: $p.PB5,
                SEED_PIN_11: $p.PB8,
                SEED_PIN_12: $p.PB9,
                SEED_PIN_13: $p.PB6,
                SEED_PIN_14: $p.PB7,
                SEED_PIN_15: $p.PC0,
                SEED_PIN_16: $p.PA3,
                SEED_PIN_17: $p.PB1,
                SEED_PIN_18: $p.PA7,
                SEED_PIN_19: $p.PA6,
                SEED_PIN_20: $p.PC1,
                SEED_PIN_21: $p.PC4,
                SEED_PIN_22: $p.PA5,
                SEED_PIN_23: $p.PA4,
                SEED_PIN_24: $p.PA1,
                SEED_PIN_25: $p.PA0,
                SEED_PIN_26: $p.PD11,
                SEED_PIN_27: $p.PG9,
                SEED_PIN_28: $p.PA2,
                SEED_PIN_29: $p.PB14,
                SEED_PIN_30: $p.PB15,
            },
            led_user_pin: $p.PC7,
            wm8731_pin: WM8731Pins {
                SCL: $p.PH4,
                SDA: $p.PB11,
                MCLK_A: $p.PE2,
                SCK_A: $p.PE5,
                FS_A: $p.PE4,
                SD_A: $p.PE6,
                SD_B: $p.PE3,
            },
            audio_peripherals: daisy_embassy::audio::Peripherals {
                sai1: $p.SAI1,
                i2c2: $p.I2C2,
                dma1_ch1: $p.DMA1_CH1,
                dma1_ch2: $p.DMA1_CH2,
            },
            usb2_pins: USB2Pins {
                DN: $p.PA11,
                DP: $p.PA12,
            },
            usb_otg_fs: $p.USB_OTG_FS,
        }
    };
}
