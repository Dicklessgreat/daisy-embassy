use embassy_stm32 as hal;
use hal::{peripherals, time::Hertz};
use static_cell::StaticCell;

use crate::{board::Irqs, pins::WM8731Pins};
// - global constants ---------------------------------------------------------

const FS: Hertz = Hertz(48000);
const I2C_FS: Hertz = Hertz(100_000);
pub const BLOCK_LENGTH: usize = 32; // 32 samples
pub const HALF_DMA_BUFFER_LENGTH: usize = BLOCK_LENGTH * 2; //  2 channels
pub const DMA_BUFFER_LENGTH: usize = HALF_DMA_BUFFER_LENGTH * 2; //  2 half-blocks

// - types --------------------------------------------------------------------

pub type Frame = (f32, f32);
pub type Block = [Frame; BLOCK_LENGTH];

pub struct Interface<'a> {
    pub fs: Hertz,
    sai_tx: hal::sai::Sai<'a, peripherals::SAI1, u32>,
    sai_rx: hal::sai::Sai<'a, peripherals::SAI1, u32>,
    i2c: hal::i2c::I2c<'a, hal::mode::Async>,
}

impl<'a> Interface<'a> {
    pub fn init(
        // clocks: &hal::rcc::CoreClocks,
        wm8731: WM8731Pins,
        sai1: hal::peripherals::SAI1, // reset and enable control
        i2c2: hal::peripherals::I2C2, // reset and enable control
        dma1: hal::peripherals::DMA1,
        dma1_ch1: hal::peripherals::DMA1_CH1,
        dma1_ch2: hal::peripherals::DMA1_CH2,
        dma1_ch4: hal::peripherals::DMA1_CH4,
        dma1_ch5: hal::peripherals::DMA1_CH5,
    ) -> Self {
        use hal::sai::{
            ClockStrobe, Config, DataSize, FifoThreshold, MasterClockDivider, Mode, StereoMono,
            TxRx,
        };
        let (sub_block_receiver, sub_block_transmitter) = hal::sai::split_subblocks(sai1);

        let mut sai_tx_config = Config::default();
        sai_tx_config.mode = Mode::Slave;
        sai_tx_config.tx_rx = TxRx::Transmitter;
        sai_tx_config.mute_detection_counter = hal::dma::word::U5(0);
        sai_tx_config.master_clock_divider = MasterClockDivider::Div12;
        sai_tx_config.fifo_threshold = FifoThreshold::Empty;
        sai_tx_config.sync_output = true;
        sai_tx_config.stereo_mono = StereoMono::Stereo;
        sai_tx_config.data_size = DataSize::Data24;
        sai_tx_config.clock_strobe = ClockStrobe::Falling;
        static TX_BUFFER: StaticCell<[u32; DMA_BUFFER_LENGTH]> = StaticCell::new();
        let tx_buffer = TX_BUFFER.init([0; DMA_BUFFER_LENGTH]);
        let sai_tx = hal::sai::Sai::new_synchronous(
            sub_block_transmitter,
            wm8731.SD_B,
            dma1_ch1,
            tx_buffer,
            sai_tx_config,
        );

        let mut sai_rx_config = Config::default();
        sai_rx_config.tx_rx = TxRx::Receiver;
        sai_rx_config.mode = Mode::Master;
        static RX_BUFFER: StaticCell<[u32; DMA_BUFFER_LENGTH]> = StaticCell::new();
        let rx_buffer = RX_BUFFER.init([0; DMA_BUFFER_LENGTH]);
        let sai_rx = hal::sai::Sai::new_asynchronous_with_mclk(
            sub_block_receiver,
            wm8731.SCK_A,
            wm8731.SD_A,
            wm8731.FS_A,
            wm8731.MCLK_A,
            dma1_ch2,
            rx_buffer,
            sai_rx_config,
        );

        let i2c_config = hal::i2c::Config::default();
        let i2c = embassy_stm32::i2c::I2c::new(
            i2c2, wm8731.SCL, wm8731.SDA, Irqs, dma1_ch4, dma1_ch5, I2C_FS, i2c_config,
        );

        Self {
            fs: FS,
            sai_rx,
            sai_tx,
            i2c,
        }
    }
}
