use embassy_stm32 as hal;
use hal::peripherals::*;

pub struct Codec;

// ToDo

#[allow(non_snake_case)]
pub struct Pins {
    // ToDo remove i2c
    pub SCL: PH4,    // I2C SCL
    pub SDA: PB11,   // I2C SDA


    pub MCLK_A: PE2, // SAI1 MCLK_A
    pub SCK_A: PE5,  // SAI1 SCK_A
    pub FS_A: PE4,   // SAI1 FS_A
    pub SD_A: PE6,   // SAI1 SD_A
    pub SD_B: PE3,   // SAI1 SD_B
}

// ToDo
