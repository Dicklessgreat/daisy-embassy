use crate::led::UserLed;
use crate::pins::*;
use embassy_stm32 as hal;

#[allow(non_snake_case)]
pub struct DaisyBoard<'a> {
    // https://github.com/electro-smith/DaisyWiki/wiki/2.-Daisy-Seed-Pinout
    pub SEED_PIN_0: SeedPin0,
    pub SEED_PIN_1: SeedPin1,
    pub SEED_PIN_2: SeedPin2,
    pub SEED_PIN_3: SeedPin3,
    pub SEED_PIN_4: SeedPin4,
    pub SEED_PIN_5: SeedPin5,
    pub SEED_PIN_6: SeedPin6,
    pub SEED_PIN_7: SeedPin7,
    pub SEED_PIN_8: SeedPin8,
    pub SEED_PIN_9: SeedPin9,
    pub SEED_PIN_10: SeedPin10,
    pub SEED_PIN_11: SeedPin11,
    pub SEED_PIN_12: SeedPin12,
    pub SEED_PIN_13: SeedPin13,
    pub SEED_PIN_14: SeedPin14,
    pub SEED_PIN_15: SeedPin15,
    pub SEED_PIN_16: SeedPin16,
    pub SEED_PIN_17: SeedPin17,
    pub SEED_PIN_18: SeedPin18,
    pub SEED_PIN_19: SeedPin19,
    pub SEED_PIN_20: SeedPin20,
    pub SEED_PIN_21: SeedPin21,
    pub SEED_PIN_22: SeedPin22,
    pub SEED_PIN_23: SeedPin23,
    pub SEED_PIN_24: SeedPin24,
    pub SEED_PIN_25: SeedPin25,
    pub SEED_PIN_26: SeedPin26,
    pub SEED_PIN_27: SeedPin27,
    pub SEED_PIN_28: SeedPin28,
    pub SEED_PIN_29: SeedPin29,
    pub SEED_PIN_30: SeedPin30,

    // board peripherals
    pub LED_USER: UserLed<'a>,
    pub WM8731: WM8731Pins,
    pub FMC: FMCPins,
    pub SDRAM: (), // TODO
    pub USB2: USB2Pins,
}

impl<'a> DaisyBoard<'a> {
    pub fn new(config: embassy_stm32::Config) -> Self {
        let p = embassy_stm32::init(config);
        Self {
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
            LED_USER: UserLed::new(p.PC7),
            WM8731: WM8731Pins {
                SCL: p.PH4,
                SDA: p.PB11,
                MCLK_A: p.PE2,
                SCK_A: p.PE5,
                FS_A: p.PE4,
                SD_A: p.PE6,
                SD_B: p.PE3,
            },
            FMC: FMCPins {
                IO0: p.PF8,
                IO1: p.PF9,
                IO2: p.PF7,
                IO3: p.PF6,
                SCK: p.PF10,
                CS: p.PG6,
            },
            SDRAM: (),
            USB2: USB2Pins {
                DN: p.PA11, // USB2 D-
                DP: p.PA12, // USB2 D+
            },
        }
    }
}
