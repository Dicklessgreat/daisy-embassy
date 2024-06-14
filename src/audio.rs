use embassy_stm32 as hal;
use hal::{dma::Transfer, peripherals, time::Hertz};
// - global constants ---------------------------------------------------------

pub const BLOCK_LENGTH: usize = 32;                             // 32 samples
pub const HALF_DMA_BUFFER_LENGTH: usize = BLOCK_LENGTH * 2;     //  2 channels
pub const DMA_BUFFER_LENGTH:usize = HALF_DMA_BUFFER_LENGTH * 2; //  2 half-blocks

// - static data --------------------------------------------------------------

#[link_section = ".sram1_bss"]
static mut TX_BUFFER: [u32; DMA_BUFFER_LENGTH] = [0; DMA_BUFFER_LENGTH];
#[link_section = ".sram1_bss"]
static mut RX_BUFFER: [u32; DMA_BUFFER_LENGTH] = [0; DMA_BUFFER_LENGTH];

// - types --------------------------------------------------------------------

pub type Frame = (f32, f32);
pub type Block = [Frame; BLOCK_LENGTH];

pub type Sai1Pins = (
    // gpio::gpiob::PB11<gpio::Output<gpio::PushPull>>,  // PDN
    peripherals::PE2,     // MCLK_A
    peripherals::PE5,     // SCK_A
    peripherals::PE4,     // FS_A
    peripherals::PE6,     // SD_A
    peripherals::PE3,     // SD_B
);

pub type I2C2Pins = (
    peripherals::PH4,  // SCL
    peripherals::PB11, // SDA
);

pub struct Interface<'a> {
    pub fs: Hertz,

    function_ptr: Option<fn(f32, &mut Block)>,

    hal_dma1_stream0: Option<Transfer<'a>>,
    hal_dma1_stream1: Option<Transfer<'a>>,
    hal_sai1: Option<hal::sai::Sai<'a, peripherals::SAI1, u8>>,
    hal_i2c2: Option<hal::i2c::I2c<'a, hal::mode::Async>>,
}