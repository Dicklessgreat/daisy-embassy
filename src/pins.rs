use embassy_stm32 as hal;
use hal::peripherals::*;

// - types --------------------------------------------------------------------

pub type SeedPin0 = PB12; // PIN_01, USB OTG ID, I2C3 SCL
pub type SeedPin1 = PC11; // PIN_02, SD Data3, USART3 Rx
pub type SeedPin2 = PC10; // PIN_03, SD Data2, USART3 Tx
pub type SeedPin3 = PC9; // PIN_04, SD Data1, I2C3 SDA
pub type SeedPin4 = PC8; // PIN_05, SD Data0
pub type SeedPin5 = PD2; // PIN_06, SD CMD, UART5 Rx
pub type SeedPin6 = PC12; // PIN_07, SD CLK, UART5 Tx
pub type SeedPin7 = PG10; // PIN_08, SPI1 CS
pub type SeedPin8 = PG11; // PIN_09, SPI1 SCK, SPDIFRX1
pub type SeedPin9 = PB4; // PIN_10, SPI1 MISO
pub type SeedPin10 = PB5; // PIN_11, SPI1 MOSI
pub type SeedPin11 = PB8; // PIN_12, I2C1 SCL, UART4 Rx
pub type SeedPin12 = PB9; // PIN_13, I2C1 SDA, UART4 Tx
pub type SeedPin13 = PB6; // PIN_14, USART1 Tx, I2C4 SCL
pub type SeedPin14 = PB7; // PIN_15, USART1 Rx, I2C4 SDA
pub type SeedPin15 = PC0; // PIN_22, ADC 0
pub type SeedPin16 = PA3; // PIN_23, ADC 1
pub type SeedPin17 = PB1; // PIN_24, ADC 2
pub type SeedPin18 = PA7; // PIN_25, ADC 3
pub type SeedPin19 = PA6; // PIN_26, ADC 4
pub type SeedPin20 = PC1; // PIN_27, ADC 5
pub type SeedPin21 = PC4; // PIN_28, ADC 6
pub type SeedPin22 = PA5; // PIN_29, DAC OUT 2, ADC 7
pub type SeedPin23 = PA4; // PIN_30, DAC OUT 1, ADC 8
pub type SeedPin24 = PA1; // PIN_31, SAI2 MCLK, ADC 9
pub type SeedPin25 = PA0; // PIN_32, SAI2 SD B, ADC 10
pub type SeedPin26 = PD11; // PIN_33, SAI2 SD A
pub type SeedPin27 = PG9; // PIN_34, SAI2 SD FS
pub type SeedPin28 = PA2; // PIN_35, SAI2 SCK, ADC 11
pub type SeedPin29 = PB14; // PIN_36, USB1 D-, USART1 Tx
pub type SeedPin30 = PB15; // PIN_37, USB1 D+, USART1 Rx

pub type Boot = PG3; //on board "BOOT" button

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

pub type LedUserPin = PC7; // LED_USER

#[allow(non_snake_case)]
pub struct WM8731Pins {
    pub SCL: PH4,    // I2C SCL
    pub SDA: PB11,   // I2C SDA
    pub MCLK_A: PE2, // SAI1 MCLK_A
    pub SCK_A: PE5,  // SAI1 SCK_A
    pub FS_A: PE4,   // SAI1 FS_A
    pub SD_A: PE6,   // SAI1 SD_A
    pub SD_B: PE3,   // SAI1 SD_B
}

#[allow(non_snake_case)]
pub struct USB2Pins {
    pub DN: PA11, // USB2 D-
    pub DP: PA12, // USB2 D+
}

#[allow(non_snake_case)]
pub struct FlashPins {
    // https://github.com/electro-smith/libDaisy/blob/3dda55e9ed55a2f8b6bc4fa6aa2c7ae134c317ab/src/per/qspi.c#L695
    pub IO0: PF8, // (SI)
    pub IO1: PF9, // (SO)
    pub IO2: PF7,
    pub IO3: PF6,
    pub SCK: PF10,
    pub CS: PG6,
}

pub struct SdRamPins {
    pub dd0: PD0,
    pub dd1: PD1,
    pub dd8: PD8,
    pub dd9: PD9,
    pub dd10: PD10,
    pub dd14: PD14,
    pub dd15: PD15,
    pub ee0: PE0,
    pub ee1: PE1,
    pub ee7: PE7,
    pub ee8: PE8,
    pub ee9: PE9,
    pub ee10: PE10,
    pub ee11: PE11,
    pub ee12: PE12,
    pub ee13: PE13,
    pub ee14: PE14,
    pub ee15: PE15,
    pub ff0: PF0,
    pub ff1: PF1,
    pub ff2: PF2,
    pub ff3: PF3,
    pub ff4: PF4,
    pub ff5: PF5,
    pub ff11: PF11,
    pub ff12: PF12,
    pub ff13: PF13,
    pub ff14: PF14,
    pub ff15: PF15,
    pub gg0: PG0,
    pub gg1: PG1,
    pub gg2: PG2,
    pub gg4: PG4,
    pub gg5: PG5,
    pub gg8: PG8,
    pub gg15: PG15,
    pub hh2: PH2,
    pub hh3: PH3,
    pub hh5: PH5,
    pub hh8: PH8,
    pub hh9: PH9,
    pub hh10: PH10,
    pub hh11: PH11,
    pub hh12: PH12,
    pub hh13: PH13,
    pub hh14: PH14,
    pub hh15: PH15,
    pub ii0: PI0,
    pub ii1: PI1,
    pub ii2: PI2,
    pub ii3: PI3,
    pub ii4: PI4,
    pub ii5: PI5,
    pub ii6: PI6,
    pub ii7: PI7,
    pub ii9: PI9,
    pub ii10: PI10,
}
