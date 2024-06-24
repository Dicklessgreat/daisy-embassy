use embassy_stm32 as hal;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    zerocopy_channel::{Receiver, Sender},
};
use embassy_time::Timer;
use hal::{
    peripherals,
    sai::{
        self, ClockStrobe, ComplementFormat, Config, DataSize, FrameSyncPolarity,
        MasterClockDivider, Mode, Sai, StereoMono, TxRx,
    },
    time::Hertz,
};
use static_cell::StaticCell;

use crate::{board::Irqs, pins::WM8731Pins};
// - global constants ---------------------------------------------------------

// const FS: Hertz = Hertz(48000);
const I2C_FS: Hertz = Hertz(100_000);
pub const BLOCK_LENGTH: usize = 32; // 32 samples
pub const HALF_DMA_BUFFER_LENGTH: usize = BLOCK_LENGTH * 2; //  2 channels
pub const DMA_BUFFER_LENGTH: usize = HALF_DMA_BUFFER_LENGTH * 2; //  2 half-blocks

// - types --------------------------------------------------------------------

pub type InterleavedBlock = [u32; BLOCK_LENGTH * 2];

pub struct Interface<'a> {
    sai_tx_conf: sai::Config,
    sai_rx_conf: sai::Config,
    sai_tx: Sai<'a, peripherals::SAI1, u32>,
    sai_rx: Sai<'a, peripherals::SAI1, u32>,
    i2c: hal::i2c::I2c<'a, hal::mode::Async>,
}

pub struct Peripherals {
    pub sai1: hal::peripherals::SAI1,
    pub i2c2: hal::peripherals::I2C2,
    pub dma1_ch1: hal::peripherals::DMA1_CH1,
    pub dma1_ch2: hal::peripherals::DMA1_CH2,
    pub dma1_ch4: hal::peripherals::DMA1_CH4,
    pub dma1_ch5: hal::peripherals::DMA1_CH5,
}

pub struct Start {
    pub if_to_client: Sender<'static, NoopRawMutex, InterleavedBlock>,
    pub client_to_if: Receiver<'static, NoopRawMutex, InterleavedBlock>,
}

pub enum Fs {
    Fs32000,
    Fs44100,
    Fs48000,
    Fs64000,
    Fs88200,
    Fs96000,
    Fs128000,
    Fs176000,
    Fs192000,
}
const CLOCK_RATIO: u32 = 256; //Not yet support oversampling.
impl Fs {
    fn into_clock_divider(self) -> MasterClockDivider {
        let fs = match self {
            Fs::Fs32000 => 32000,
            Fs::Fs44100 => 44100,
            Fs::Fs48000 => 48000,
            Fs::Fs64000 => 64000,
            Fs::Fs88200 => 88200,
            Fs::Fs96000 => 96000,
            Fs::Fs128000 => 128000,
            Fs::Fs176000 => 176000,
            Fs::Fs192000 => 192000,
        };
        let kernel_clock = hal::rcc::frequency::<hal::peripherals::SAI1>().0;
        let mclk_div = (kernel_clock / (fs * CLOCK_RATIO)) as u8;
        mclk_div_from_u8(mclk_div)
    }
}

impl<'a> Interface<'a> {
    pub fn new(wm8731: WM8731Pins, p: Peripherals, tx_fs: Fs, rx_fs: Fs) -> Self {
        let (sub_block_receiver, sub_block_transmitter) = hal::sai::split_subblocks(p.sai1);

        // I have no idea how to set up SAI! WIP
        let mut sai_tx_conf = Config::default();
        sai_tx_conf.mode = Mode::Slave;
        sai_tx_conf.tx_rx = TxRx::Transmitter;
        sai_tx_conf.stereo_mono = StereoMono::Stereo;
        sai_tx_conf.data_size = DataSize::Data24;
        sai_tx_conf.clock_strobe = ClockStrobe::Falling;
        sai_tx_conf.frame_sync_polarity = FrameSyncPolarity::ActiveHigh;
        sai_tx_conf.master_clock_divider = tx_fs.into_clock_divider();
        // stm32h7xx-hal set complement format as "Ones" by default. But I don't know this matters or not.
        // sai_tx_conf.complement_format = ComplementFormat::OnesComplement;
        static TX_BUFFER: StaticCell<[u32; DMA_BUFFER_LENGTH]> = StaticCell::new();
        let tx_buffer = TX_BUFFER.init([0; DMA_BUFFER_LENGTH]);
        let sai_tx = hal::sai::Sai::new_synchronous(
            sub_block_transmitter,
            wm8731.SD_B,
            p.dma1_ch1,
            tx_buffer,
            sai_tx_conf,
        );

        // I have no idea how to set up SAI! WIP
        let mut sai_rx_conf = Config::default();
        sai_rx_conf.tx_rx = TxRx::Receiver;
        sai_rx_conf.mode = Mode::Master;
        sai_rx_conf.stereo_mono = StereoMono::Stereo;
        sai_rx_conf.data_size = DataSize::Data24;
        sai_rx_conf.clock_strobe = ClockStrobe::Rising;
        sai_rx_conf.frame_sync_polarity = FrameSyncPolarity::ActiveHigh;
        sai_rx_conf.master_clock_divider = rx_fs.into_clock_divider();
        // stm32h7xx-hal set complement format as "Ones" by default. But I don't know this matters or not.
        // sai_rx_conf.complement_format = ComplementFormat::OnesComplement;
        static RX_BUFFER: StaticCell<[u32; DMA_BUFFER_LENGTH]> = StaticCell::new();
        let rx_buffer = RX_BUFFER.init([0; DMA_BUFFER_LENGTH]);
        let sai_rx = hal::sai::Sai::new_asynchronous_with_mclk(
            sub_block_receiver,
            wm8731.SCK_A,
            wm8731.SD_A,
            wm8731.FS_A,
            wm8731.MCLK_A,
            p.dma1_ch2,
            rx_buffer,
            sai_rx_conf,
        );

        let i2c_config = hal::i2c::Config::default();
        let i2c = embassy_stm32::i2c::I2c::new(
            p.i2c2, wm8731.SCL, wm8731.SDA, Irqs, p.dma1_ch4, p.dma1_ch5, I2C_FS, i2c_config,
        );

        Self {
            sai_rx_conf,
            sai_tx_conf,
            sai_rx,
            sai_tx,
            i2c,
        }
    }
    pub async fn start<S: Send>(&mut self, start: Start) -> ! {
        // - set up WM8731 ------------------------------------------------------
        // from https://github.com/backtail/daisy_bsp/blob/b7b80f78dafc837b90e97a265d2a3378094b84f7/src/audio.rs#L234C9-L235C1
        let codec_i2c_address: u8 = 0x1a; // or 0x1b if CSB is high

        // Go through configuration setup
        for (register, value) in REGISTER_CONFIG {
            let register = *register as u8;
            let value = *value;
            let byte1: u8 = ((register << 1) & 0b1111_1110) | ((value >> 7) & 0b0000_0001u8);
            let byte2: u8 = value;
            let bytes = [byte1, byte2];

            self.i2c.write(codec_i2c_address, &bytes).await.unwrap();

            // wait ~10us
            Timer::after_micros(10).await;
        }

        // - start audio ------------------------------------------------------

        self.sai_tx.start();
        self.sai_rx.start();
        // in daisy_bsp/src/audio.rs...Interface::start(), it waits untill sai1's fifo starts to receive data.
        // I don't know how to get fifo state in embassy.
        let Start {
            mut if_to_client,
            mut client_to_if,
        } = start;

        loop {
            // Obtain a free buffer from the channel
            let buf = if_to_client.send().await;
            // and fill it with data
            self.sai_rx.read(buf).await.unwrap();
            //Notify the channel that the buffer is now ready to be received
            if_to_client.send_done();
            // await till client audio callback task finish processing
            let buf = client_to_if.receive().await;
            self.sai_tx.write(buf).await.unwrap();
            self.sai_tx.flush();
            client_to_if.receive_done();
        }
    }
    pub fn rx_config(&self) -> &sai::Config {
        &self.sai_rx_conf
    }
    pub fn tx_config(&self) -> &sai::Config {
        &self.sai_tx_conf
    }
}

//from https://github.com/backtail/daisy_bsp/blob/b7b80f78dafc837b90e97a265d2a3378094b84f7/src/audio.rs#L381
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum Register {
    LINVOL = 0x00,
    RINVOL = 0x01,
    LOUT1V = 0x02,
    ROUT1V = 0x03,
    APANA = 0x04,
    APDIGI = 0x05, // 0000_0101
    PWR = 0x06,
    IFACE = 0x07,  // 0000_0111
    SRATE = 0x08,  // 0000_1000
    ACTIVE = 0x09, // 0000_1001
    RESET = 0x0F,
}
const REGISTER_CONFIG: &[(Register, u8)] = &[
    // reset Codec
    (Register::RESET, 0x00),
    // set line inputs 0dB
    (Register::LINVOL, 0x17),
    (Register::RINVOL, 0x17),
    // set headphone to mute
    (Register::LOUT1V, 0x00),
    (Register::ROUT1V, 0x00),
    // set analog and digital routing
    (Register::APANA, 0x12),
    (Register::APDIGI, 0x01),
    // configure power management
    (Register::PWR, 0x42),
    // configure digital format
    (Register::IFACE, 0x0A),
    // set samplerate
    (Register::SRATE, 0x00),
    (Register::ACTIVE, 0x00),
    (Register::ACTIVE, 0x01),
];

const fn mclk_div_from_u8(v: u8) -> MasterClockDivider {
    match v {
        1 => MasterClockDivider::Div1,
        2 => MasterClockDivider::Div2,
        3 => MasterClockDivider::Div3,
        4 => MasterClockDivider::Div4,
        5 => MasterClockDivider::Div5,
        6 => MasterClockDivider::Div6,
        7 => MasterClockDivider::Div7,
        8 => MasterClockDivider::Div8,
        9 => MasterClockDivider::Div9,
        10 => MasterClockDivider::Div10,
        11 => MasterClockDivider::Div11,
        12 => MasterClockDivider::Div12,
        13 => MasterClockDivider::Div13,
        14 => MasterClockDivider::Div14,
        15 => MasterClockDivider::Div15,
        16 => MasterClockDivider::Div16,
        17 => MasterClockDivider::Div17,
        18 => MasterClockDivider::Div18,
        19 => MasterClockDivider::Div19,
        20 => MasterClockDivider::Div20,
        21 => MasterClockDivider::Div21,
        22 => MasterClockDivider::Div22,
        23 => MasterClockDivider::Div23,
        24 => MasterClockDivider::Div24,
        25 => MasterClockDivider::Div25,
        26 => MasterClockDivider::Div26,
        27 => MasterClockDivider::Div27,
        28 => MasterClockDivider::Div28,
        29 => MasterClockDivider::Div29,
        30 => MasterClockDivider::Div30,
        31 => MasterClockDivider::Div31,
        32 => MasterClockDivider::Div32,
        33 => MasterClockDivider::Div33,
        34 => MasterClockDivider::Div34,
        35 => MasterClockDivider::Div35,
        36 => MasterClockDivider::Div36,
        37 => MasterClockDivider::Div37,
        38 => MasterClockDivider::Div38,
        39 => MasterClockDivider::Div39,
        40 => MasterClockDivider::Div40,
        41 => MasterClockDivider::Div41,
        42 => MasterClockDivider::Div42,
        43 => MasterClockDivider::Div43,
        44 => MasterClockDivider::Div44,
        45 => MasterClockDivider::Div45,
        46 => MasterClockDivider::Div46,
        47 => MasterClockDivider::Div47,
        48 => MasterClockDivider::Div48,
        49 => MasterClockDivider::Div49,
        50 => MasterClockDivider::Div50,
        51 => MasterClockDivider::Div51,
        52 => MasterClockDivider::Div52,
        53 => MasterClockDivider::Div53,
        54 => MasterClockDivider::Div54,
        55 => MasterClockDivider::Div55,
        56 => MasterClockDivider::Div56,
        57 => MasterClockDivider::Div57,
        58 => MasterClockDivider::Div58,
        59 => MasterClockDivider::Div59,
        60 => MasterClockDivider::Div60,
        61 => MasterClockDivider::Div61,
        62 => MasterClockDivider::Div62,
        63 => MasterClockDivider::Div63,
        _ => panic!(),
    }
}
