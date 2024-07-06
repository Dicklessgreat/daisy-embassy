use embassy_stm32 as hal;
use hal::peripherals;

// - types --------------------------------------------------------------------

pub type SeedPin0 = peripherals::PB12; // PIN_01, USB OTG ID, I2C3 SCL
pub type SeedPin1 = peripherals::PC11; // PIN_02, SD Data3, USART3 Rx
pub type SeedPin2 = peripherals::PC10; // PIN_03, SD Data2, USART3 Tx
pub type SeedPin3 = peripherals::PC9; // PIN_04, SD Data1, I2C3 SDA
pub type SeedPin4 = peripherals::PC8; // PIN_05, SD Data0
pub type SeedPin5 = peripherals::PD2; // PIN_06, SD CMD, UART5 Rx
pub type SeedPin6 = peripherals::PC12; // PIN_07, SD CLK, UART5 Tx
pub type SeedPin7 = peripherals::PG10; // PIN_08, SPI1 CS
pub type SeedPin8 = peripherals::PG11; // PIN_09, SPI1 SCK, SPDIFRX1
pub type SeedPin9 = peripherals::PB4; // PIN_10, SPI1 MISO
pub type SeedPin10 = peripherals::PB5; // PIN_11, SPI1 MOSI
pub type SeedPin11 = peripherals::PB8; // PIN_12, I2C1 SCL, UART4 Rx
pub type SeedPin12 = peripherals::PB9; // PIN_13, I2C1 SDA, UART4 Tx
pub type SeedPin13 = peripherals::PB6; // PIN_14, USART1 Tx, I2C4 SCL
pub type SeedPin14 = peripherals::PB7; // PIN_15, USART1 Rx, I2C4 SDA
pub type SeedPin15 = peripherals::PC0; // PIN_22, ADC 0
pub type SeedPin16 = peripherals::PA3; // PIN_23, ADC 1
pub type SeedPin17 = peripherals::PB1; // PIN_24, ADC 2
pub type SeedPin18 = peripherals::PA7; // PIN_25, ADC 3
pub type SeedPin19 = peripherals::PA6; // PIN_26, ADC 4
pub type SeedPin20 = peripherals::PC1; // PIN_27, ADC 5
pub type SeedPin21 = peripherals::PC4; // PIN_28, ADC 6
pub type SeedPin22 = peripherals::PA5; // PIN_29, DAC OUT 2, ADC 7
pub type SeedPin23 = peripherals::PA4; // PIN_30, DAC OUT 1, ADC 8
pub type SeedPin24 = peripherals::PA1; // PIN_31, SAI2 MCLK, ADC 9
pub type SeedPin25 = peripherals::PA0; // PIN_32, SAI2 SD B, ADC 10
pub type SeedPin26 = peripherals::PD11; // PIN_33, SAI2 SD A
pub type SeedPin27 = peripherals::PG9; // PIN_34, SAI2 SD FS
pub type SeedPin28 = peripherals::PA2; // PIN_35, SAI2 SCK, ADC 11
pub type SeedPin29 = peripherals::PB14; // PIN_36, USB1 D-, USART1 Tx
pub type SeedPin30 = peripherals::PB15; // PIN_37, USB1 D+, USART1 Rx

pub type Boot = peripherals::PG3; //on board "BOOT" button

pub struct DaisyPins {
    pub d0: SeedPin0,
    pub d1: SeedPin1,
    pub d2: SeedPin2,
    pub d3: SeedPin3,
    pub d4: SeedPin4,
    pub d5: SeedPin5,
    pub d6: SeedPin6,
    pub d7: SeedPin7,
    pub d8: SeedPin8,
    pub d9: SeedPin9,
    pub d10: SeedPin10,
    pub d11: SeedPin11,
    pub d12: SeedPin12,
    pub d13: SeedPin13,
    pub d14: SeedPin14,
    pub d15: SeedPin15,
    pub d16: SeedPin16,
    pub d17: SeedPin17,
    pub d18: SeedPin18,
    pub d19: SeedPin19,
    pub d20: SeedPin20,
    pub d21: SeedPin21,
    pub d22: SeedPin22,
    pub d23: SeedPin23,
    pub d24: SeedPin24,
    pub d25: SeedPin25,
    pub d26: SeedPin26,
    pub d27: SeedPin27,
    pub d28: SeedPin28,
    pub d29: SeedPin29,
    pub d30: SeedPin30,
}

pub type LedUserPin = peripherals::PC7; // LED_USER

#[allow(non_snake_case)]
pub struct WM8731Pins {
    pub SCL: peripherals::PH4,    // I2C SCL
    pub SDA: peripherals::PB11,   // I2C SDA
    pub MCLK_A: peripherals::PE2, // SAI1 MCLK_A
    pub SCK_A: peripherals::PE5,  // SAI1 SCK_A
    pub FS_A: peripherals::PE4,   // SAI1 FS_A
    pub SD_A: peripherals::PE6,   // SAI1 SD_A
    pub SD_B: peripherals::PE3,   // SAI1 SD_B
}

#[allow(non_snake_case)]
pub struct USB2Pins {
    pub DN: peripherals::PA11, // USB2 D-
    pub DP: peripherals::PA12, // USB2 D+
}

#[allow(non_snake_case)]
pub struct FMCPins {
    // https://github.com/electro-smith/libDaisy/blob/3dda55e9ed55a2f8b6bc4fa6aa2c7ae134c317ab/src/per/qspi.c#L695
    pub IO0: peripherals::PF8, // (SI)
    pub IO1: peripherals::PF9, // (SO)
    pub IO2: peripherals::PF7,
    pub IO3: peripherals::PF6,
    pub SCK: peripherals::PF10,
    pub CS: peripherals::PG6,
}

pub struct SdRamPins {
    pub dd0: peripherals::PD0,
    pub dd1: peripherals::PD1,
    pub dd8: peripherals::PD8,
    pub dd9: peripherals::PD9,
    pub dd10: peripherals::PD10,
    pub dd14: peripherals::PD14,
    pub dd15: peripherals::PD15,
    pub ee0: peripherals::PE0,
    pub ee1: peripherals::PE1,
    pub ee7: peripherals::PE7,
    pub ee8: peripherals::PE8,
    pub ee9: peripherals::PE9,
    pub ee10: peripherals::PE10,
    pub ee11: peripherals::PE11,
    pub ee12: peripherals::PE12,
    pub ee13: peripherals::PE13,
    pub ee14: peripherals::PE14,
    pub ee15: peripherals::PE15,
    pub ff0: peripherals::PF0,
    pub ff1: peripherals::PF1,
    pub ff2: peripherals::PF2,
    pub ff3: peripherals::PF3,
    pub ff4: peripherals::PF4,
    pub ff5: peripherals::PF5,
    pub ff11: peripherals::PF11,
    pub ff12: peripherals::PF12,
    pub ff13: peripherals::PF13,
    pub ff14: peripherals::PF14,
    pub ff15: peripherals::PF15,
    pub gg0: peripherals::PG0,
    pub gg1: peripherals::PG1,
    pub gg2: peripherals::PG2,
    pub gg4: peripherals::PG4,
    pub gg5: peripherals::PG5,
    pub gg8: peripherals::PG8,
    pub gg15: peripherals::PG15,
    pub hh2: peripherals::PH2,
    pub hh3: peripherals::PH3,
    pub hh5: peripherals::PH5,
    pub hh8: peripherals::PH8,
    pub hh9: peripherals::PH9,
    pub hh10: peripherals::PH10,
    pub hh11: peripherals::PH11,
    pub hh12: peripherals::PH12,
    pub hh13: peripherals::PH13,
    pub hh14: peripherals::PH14,
    pub hh15: peripherals::PH15,
    pub ii0: peripherals::PI0,
    pub ii1: peripherals::PI1,
    pub ii2: peripherals::PI2,
    pub ii3: peripherals::PI3,
    pub ii4: peripherals::PI4,
    pub ii5: peripherals::PI5,
    pub ii6: peripherals::PI6,
    pub ii7: peripherals::PI7,
    pub ii9: peripherals::PI9,
    pub ii10: peripherals::PI10,
}
