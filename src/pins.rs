use embassy_stm32 as hal;

// - types --------------------------------------------------------------------

pub type SeedPin0 = hal::peripherals::PB12; // PIN_01, USB OTG ID, I2C3 SCL
pub type SeedPin1 = hal::peripherals::PC11; // PIN_02, SD Data3, USART3 Rx
pub type SeedPin2 = hal::peripherals::PC10; // PIN_03, SD Data2, USART3 Tx
pub type SeedPin3 = hal::peripherals::PC9; // PIN_04, SD Data1, I2C3 SDA
pub type SeedPin4 = hal::peripherals::PC8; // PIN_05, SD Data0
pub type SeedPin5 = hal::peripherals::PD2; // PIN_06, SD CMD, UART5 Rx
pub type SeedPin6 = hal::peripherals::PC12; // PIN_07, SD CLK, UART5 Tx
pub type SeedPin7 = hal::peripherals::PG10; // PIN_08, SPI1 CS
pub type SeedPin8 = hal::peripherals::PG11; // PIN_09, SPI1 SCK, SPDIFRX1
pub type SeedPin9 = hal::peripherals::PB4; // PIN_10, SPI1 MISO
pub type SeedPin10 = hal::peripherals::PB5; // PIN_11, SPI1 MOSI
pub type SeedPin11 = hal::peripherals::PB8; // PIN_12, I2C1 SCL, UART4 Rx
pub type SeedPin12 = hal::peripherals::PB9; // PIN_13, I2C1 SDA, UART4 Tx
pub type SeedPin13 = hal::peripherals::PB6; // PIN_14, USART1 Tx, I2C4 SCL
pub type SeedPin14 = hal::peripherals::PB7; // PIN_15, USART1 Rx, I2C4 SDA
pub type SeedPin15 = hal::peripherals::PC0; // PIN_22, ADC 0
pub type SeedPin16 = hal::peripherals::PA3; // PIN_23, ADC 1
pub type SeedPin17 = hal::peripherals::PB1; // PIN_24, ADC 2
pub type SeedPin18 = hal::peripherals::PA7; // PIN_25, ADC 3
pub type SeedPin19 = hal::peripherals::PA6; // PIN_26, ADC 4
pub type SeedPin20 = hal::peripherals::PC1; // PIN_27, ADC 5
pub type SeedPin21 = hal::peripherals::PC4; // PIN_28, ADC 6
pub type SeedPin22 = hal::peripherals::PA5; // PIN_29, DAC OUT 2, ADC 7
pub type SeedPin23 = hal::peripherals::PA4; // PIN_30, DAC OUT 1, ADC 8
pub type SeedPin24 = hal::peripherals::PA1; // PIN_31, SAI2 MCLK, ADC 9
pub type SeedPin25 = hal::peripherals::PA0; // PIN_32, SAI2 SD B, ADC 10
pub type SeedPin26 = hal::peripherals::PD11; // PIN_33, SAI2 SD A
pub type SeedPin27 = hal::peripherals::PG9; // PIN_34, SAI2 SD FS
pub type SeedPin28 = hal::peripherals::PA2; // PIN_35, SAI2 SCK, ADC 11
pub type SeedPin29 = hal::peripherals::PB14; // PIN_36, USB1 D-, USART1 Tx
pub type SeedPin30 = hal::peripherals::PB15; // PIN_37, USB1 D+, USART1 Rx

pub type LedUserPin = hal::peripherals::PC7; // LED_USER

#[allow(non_snake_case)]
pub struct WM8731Pins {
    pub SCL: hal::peripherals::PH4,    // I2C SCL
    pub SDA: hal::peripherals::PB11,   // I2C SDA
    pub MCLK_A: hal::peripherals::PE2, // SAI1 MCLK_A
    pub SCK_A: hal::peripherals::PE5,  // SAI1 SCK_A
    pub FS_A: hal::peripherals::PE4,   // SAI1 FS_A
    pub SD_A: hal::peripherals::PE6,   // SAI1 SD_A
    pub SD_B: hal::peripherals::PE3,   // SAI1 SD_B
}

#[allow(non_snake_case)]
pub struct USB2Pins {
    pub DN: hal::peripherals::PA11, // USB2 D-
    pub DP: hal::peripherals::PA12, // USB2 D+
}

#[allow(non_snake_case)]
pub struct FMCPins {
    // https://github.com/electro-smith/libDaisy/blob/3dda55e9ed55a2f8b6bc4fa6aa2c7ae134c317ab/src/per/qspi.c#L695
    pub IO0: hal::peripherals::PF8, // (SI)
    pub IO1: hal::peripherals::PF9, // (SO)
    pub IO2: hal::peripherals::PF7,
    pub IO3: hal::peripherals::PF6,
    pub SCK: hal::peripherals::PF10,
    pub CS: hal::peripherals::PG6,
}

// - Pins ---------------------------------------------------------------------

#[allow(non_snake_case)]
pub struct Pins {
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
    pub LED_USER: LedUserPin,
    // pub AK4556: AK4556Pins,
    pub FMC: FMCPins,
    pub SDRAM: (), // TODO
    pub USB2: USB2Pins,
}
