use embassy_stm32 as hal;
use hal::peripherals::*;

/// Codec and Pins for the PCM3060 audio codec
/// The codec will stay empty since PCM3060 is configured in 'hardware mode'
/// via its config pins
pub struct Codec;

#[allow(non_snake_case)]
pub struct Pins {
    pub MCLK_A: PE2, // SAI1 MCLK_A
    pub SCK_A: PE5,  // SAI1 SCK_A
    pub FS_A: PE4,   // SAI1 FS_A
    pub SD_A: PE6,   // SAI1 SD_A
    pub SD_B: PE3,   // SAI1 SD_B
}

// ToDo
