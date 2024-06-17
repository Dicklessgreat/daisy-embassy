use embassy_stm32 as hal;
use hal::{dma::Transfer, peripherals, time::Hertz};
// - global constants ---------------------------------------------------------

const FS: Hertz = embassy_stm32::time::Hertz(48000); 
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

    hal_dma1_stream0: Transfer<'a>,
    hal_dma1_stream1: Transfer<'a>,
    hal_sai1: hal::sai::Sai<'a, peripherals::SAI1, u8>,
    hal_i2c2: hal::i2c::I2c<'a, hal::mode::Async>,
}

impl <'a> Interface<'a> {
    pub fn init(
        clocks: &hal::rcc::CoreClocks,
        sai1_rec: hal::rcc::rec::Sai1, // reset and enable control
        sai1_pins: Sai1Pins,
        i2c2_rec: hal::rcc::rec::I2c2, // reset and enable control
        i2c2_pins: I2C2Pins,
        dma1_rec: hal::rcc::rec::Dma1
    ) -> Result<Interface<'a>, Error> {
        use hal::sai::{ClockStrobe, Config, DataSize};
        let mut sai_config = Config::new();
        sai_config.data_size = DataSize::Data24;
        sai_config.clock_strobe = ClockStrobe::Falling;
        hal::sai::Sai::new_asynchronous(peri, sck, sd, fs, dma, dma_buf, sai_config);
        // - configure dma1 ---------------------------------------------------

        let dma1_streams = dma::dma::StreamsTuple::new(unsafe { pac::Peripherals::steal().DMA1 }, dma1_rec);

        // dma1 stream 0
        let rx_buffer: &'static mut [u32; DMA_BUFFER_LENGTH] = unsafe { &mut RX_BUFFER };
        let dma_config = dma::dma::DmaConfig::default()
            .priority(dma::config::Priority::High)
            .memory_increment(true)
            .peripheral_increment(false)
            .circular_buffer(true)
            .fifo_enable(false);

        // is later overwritten to be a P2M stream! (HAL doesn't support this yet)
        let dma1_str0: dma::Transfer<_, _, dma::MemoryToPeripheral, _, _> = dma::Transfer::init(
            dma1_streams.0,
            unsafe { pac::Peripherals::steal().SAI1 },
            rx_buffer,
            None,
            dma_config,
        );

        // dma1 stream 1
        let tx_buffer: &'static mut [u32; DMA_BUFFER_LENGTH] = unsafe { &mut TX_BUFFER };
        let dma_config = dma_config.transfer_complete_interrupt(true)
                                   .half_transfer_interrupt(true);

        // is later overwritten to be a M2P stream! (HAL doesn't support this yet)
        let dma1_str1: dma::Transfer<_, _, dma::PeripheralToMemory, _, _> = dma::Transfer::init(
            dma1_streams.1,
            unsafe { pac::Peripherals::steal().SAI1 },
            tx_buffer,
            None,
            dma_config,
        );

        // - configure sai1 ---------------------------------------------------

        let sai1_rx_config = sai::I2SChanConfig::new(sai::I2SDir::Rx)
            .set_frame_sync_active_high(true)
            .set_clock_strobe(sai::I2SClockStrobe::Falling);

        let sai1_tx_config = sai::I2SChanConfig::new(sai::I2SDir::Tx)
            .set_sync_type(sai::I2SSync::Internal)
            .set_frame_sync_active_high(true)
            .set_clock_strobe(sai::I2SClockStrobe::Rising);

        let sai1_pins = (
            sai1_pins.0,
            sai1_pins.1,
            sai1_pins.2,
            sai1_pins.3,
            Some(sai1_pins.4),
        );

        let sai1 = unsafe { pac::Peripherals::steal().SAI1 }.i2s_ch_a(
            sai1_pins,
            FS,
            sai::I2SDataSize::BITS_24,
            sai1_rec,
            clocks,
            I2sUsers::new(sai1_rx_config).add_slave(sai1_tx_config),
        );

        // manually configure Channel B as transmit stream
        let dma1_reg = unsafe { pac::Peripherals::steal().DMA1 };
        dma1_reg.st[0].cr.modify(|_ , w | w.dir().peripheral_to_memory());
        // manually configure Channel A as receive stream
        dma1_reg.st[1].cr.modify(|_ , w | w.dir().memory_to_peripheral());

        // - configure i2c ---------------------------------------------------

        let i2c2 = i2c::I2cExt::i2c(
            unsafe { pac::Peripherals::steal().I2C2 },
            i2c2_pins,
            I2C_FS,
            i2c2_rec,
            clocks,
        );

        Ok(Self {
            fs: FS,

            function_ptr: None,

            // ak4556_reset: Some(pins.0),
            hal_dma1_stream0: Some(dma1_str0),
            hal_dma1_stream1: Some(dma1_str1),
            hal_sai1: Some(sai1),
            hal_i2c2: Some(i2c2),

        })
    }
}