use crate::pins::WM8731Pins;
use defmt::info;
use defmt::unwrap;
use embassy_stm32 as hal;
use embassy_time::Timer;
use grounded::uninit::GroundedArrayCell;
use hal::sai::FifoThreshold;
use hal::sai::FrameSyncOffset;
use hal::sai::{BitOrder, SyncInput};
use hal::{
    peripherals,
    sai::{
        self, ClockStrobe, Config, DataSize, FrameSyncPolarity, MasterClockDivider, Mode, Sai,
        StereoMono, TxRx,
    },
    time::Hertz,
};
// - global constants ---------------------------------------------------------

const I2C_FS: Hertz = Hertz(100_000);
pub const BLOCK_LENGTH: usize = 32; // 32 samples
pub const HALF_DMA_BUFFER_LENGTH: usize = BLOCK_LENGTH * 2; //  2 channels
pub const DMA_BUFFER_LENGTH: usize = HALF_DMA_BUFFER_LENGTH * 2; //  2 half-blocks

// - static data --------------------------------------------------------------

//DMA buffer must be in special region. Refer https://embassy.dev/book/#_stm32_bdma_only_working_out_of_some_ram_regions
#[link_section = ".sram1_bss"]
static mut TX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();
#[link_section = ".sram1_bss"]
static mut RX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();

// - types --------------------------------------------------------------------

pub type InterleavedBlock = [u32; HALF_DMA_BUFFER_LENGTH];
pub struct AudioPeripherals {
    pub wm8731: WM8731Pins,
    pub sai1: hal::peripherals::SAI1,
    pub i2c2: hal::peripherals::I2C2,
    pub dma1_ch1: hal::peripherals::DMA1_CH1,
    pub dma1_ch2: hal::peripherals::DMA1_CH2,
}

impl AudioPeripherals {
    pub async fn prepare_interface<'a>(self, audio_config: AudioConfig) -> Interface<'a> {
        info!("set up i2c");
        let i2c_config = hal::i2c::Config::default();
        let mut i2c = embassy_stm32::i2c::I2c::new_blocking(
            self.i2c2,
            self.wm8731.SCL,
            self.wm8731.SDA,
            I2C_FS,
            i2c_config,
        );
        info!("set up WM8731");
        setup_wm8731(&mut i2c, audio_config.fs).await;

        info!("set up sai_tx");
        let (sub_block_receiver, sub_block_transmitter) = hal::sai::split_subblocks(self.sai1);

        let mut sai_rx_config = Config::default();
        sai_rx_config.mode = Mode::Master;
        sai_rx_config.tx_rx = TxRx::Receiver;
        sai_rx_config.sync_output = true;
        sai_rx_config.clock_strobe = ClockStrobe::Falling;
        sai_rx_config.master_clock_divider = audio_config.fs.into_clock_divider();
        sai_rx_config.stereo_mono = StereoMono::Stereo;
        sai_rx_config.data_size = DataSize::Data24;
        sai_rx_config.bit_order = BitOrder::MsbFirst;
        sai_rx_config.frame_sync_polarity = FrameSyncPolarity::ActiveHigh;
        sai_rx_config.frame_sync_offset = FrameSyncOffset::OnFirstBit;
        sai_rx_config.frame_length = 64;
        sai_rx_config.frame_sync_active_level_length = embassy_stm32::sai::word::U7(32);
        sai_rx_config.fifo_threshold = FifoThreshold::Quarter;

        let mut sai_tx_config = sai_rx_config;
        sai_tx_config.mode = Mode::Slave;
        sai_tx_config.tx_rx = TxRx::Transmitter;
        sai_tx_config.sync_input = SyncInput::Internal;
        sai_tx_config.clock_strobe = ClockStrobe::Rising;
        sai_tx_config.sync_output = false;

        let tx_buffer: &mut [u32] = unsafe {
            TX_BUFFER.initialize_all_copied(0);
            let (ptr, len) = TX_BUFFER.get_ptr_len();
            core::slice::from_raw_parts_mut(ptr, len)
        };
        let sai_tx = hal::sai::Sai::new_synchronous(
            sub_block_transmitter,
            self.wm8731.SD_B,
            self.dma1_ch1,
            tx_buffer,
            sai_tx_config,
        );

        let rx_buffer: &mut [u32] = unsafe {
            RX_BUFFER.initialize_all_copied(0);
            let (ptr, len) = RX_BUFFER.get_ptr_len();
            core::slice::from_raw_parts_mut(ptr, len)
        };
        let sai_rx = hal::sai::Sai::new_asynchronous_with_mclk(
            sub_block_receiver,
            self.wm8731.SCK_A,
            self.wm8731.SD_A,
            self.wm8731.FS_A,
            self.wm8731.MCLK_A,
            self.dma1_ch2,
            rx_buffer,
            sai_rx_config,
        );

        Interface {
            sai_rx_config,
            sai_tx_config,
            sai_rx,
            sai_tx,
            i2c,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Fs {
    Fs8000,
    Fs32000,
    Fs44100,
    Fs48000,
    Fs88200,
    Fs96000,
}
const CLOCK_RATIO: u32 = 256; //Not yet support oversampling.
impl Fs {
    fn into_clock_divider(self) -> MasterClockDivider {
        let fs = match self {
            Fs::Fs8000 => 8000,
            Fs::Fs32000 => 32000,
            Fs::Fs44100 => 44100,
            Fs::Fs48000 => 48000,
            Fs::Fs88200 => 88200,
            Fs::Fs96000 => 96000,
        };
        let kernel_clock = hal::rcc::frequency::<hal::peripherals::SAI1>().0;
        let mclk_div = (kernel_clock / (fs * CLOCK_RATIO)) as u8;
        mclk_div_from_u8(mclk_div)
    }
}

pub struct AudioConfig {
    fs: Fs,
}

impl Default for AudioConfig {
    fn default() -> Self {
        AudioConfig { fs: Fs::Fs48000 }
    }
}

pub struct Interface<'a> {
    sai_tx_config: sai::Config,
    sai_rx_config: sai::Config,
    sai_tx: Sai<'a, peripherals::SAI1, u32>,
    sai_rx: Sai<'a, peripherals::SAI1, u32>,
    i2c: hal::i2c::I2c<'a, hal::mode::Blocking>,
}

impl<'a> Interface<'a> {
    pub async fn start(&mut self, mut callback: impl FnMut(&[u32], &mut [u32])) -> ! {
        self.setup().await;
        info!("enter audio callback loop");
        loop {
            let mut write_buf = [0; HALF_DMA_BUFFER_LENGTH];
            let mut read_buf = [0; HALF_DMA_BUFFER_LENGTH];
            unwrap!(self.sai_rx.read(&mut read_buf).await);
            callback(&read_buf, &mut write_buf);
            unwrap!(self.sai_tx.write(&write_buf).await);
        }
    }
    pub fn rx_config(&self) -> &sai::Config {
        &self.sai_rx_config
    }
    pub fn tx_config(&self) -> &sai::Config {
        &self.sai_tx_config
    }
    // returns (sai_tx, sai_rx, i2c)
    pub async fn setup_and_release(
        mut self,
    ) -> (
        Sai<'a, peripherals::SAI1, u32>,
        Sai<'a, peripherals::SAI1, u32>,
        hal::i2c::I2c<'a, hal::mode::Blocking>,
    ) {
        self.setup().await;
        (self.sai_tx, self.sai_rx, self.i2c)
    }

    async fn setup(&mut self) {
        info!("setup WM8731");
        write_wm8731_reg(
            &mut self.i2c,
            wm8731::WM8731::power_down(final_power_settings),
        );
        Timer::after_micros(10).await;

        info!("start SAI");
        self.sai_tx.start();
        self.sai_rx.start();
    }
}

//====================wm8731 register set up functions============================
async fn setup_wm8731<'a>(i2c: &mut hal::i2c::I2c<'a, hal::mode::Blocking>, fs: Fs) {
    use wm8731::WM8731;
    info!("setup wm8731 from I2C");

    Timer::after_micros(10).await;

    // reset
    write_wm8731_reg(i2c, WM8731::reset());
    Timer::after_micros(10).await;

    // wakeup
    write_wm8731_reg(
        i2c,
        WM8731::power_down(|w| {
            final_power_settings(w);
            //output off before start()
            w.output().power_off();
        }),
    );
    Timer::after_micros(10).await;

    // disable input mute, set to 0dB gain
    write_wm8731_reg(
        i2c,
        WM8731::left_line_in(|w| {
            w.both().enable();
            w.mute().disable();
            w.volume().nearest_dB(0);
        }),
    );
    Timer::after_micros(10).await;

    // sidetone off; DAC selected; bypass off; line input selected; mic muted; mic boost off
    write_wm8731_reg(
        i2c,
        WM8731::analog_audio_path(|w| {
            w.sidetone().disable();
            w.dac_select().select();
            w.bypass().disable();
            w.input_select().line_input();
            w.mute_mic().enable();
            w.mic_boost().disable();
        }),
    );
    Timer::after_micros(10).await;

    // disable DAC mute, deemphasis for 48k
    write_wm8731_reg(
        i2c,
        WM8731::digital_audio_path(|w| {
            w.dac_mut().disable();
            w.deemphasis().frequency_48();
        }),
    );
    Timer::after_micros(10).await;

    // nothing inverted, slave, 24-bits, MSB format
    write_wm8731_reg(
        i2c,
        WM8731::digital_audio_interface_format(|w| {
            w.bit_clock_invert().no_invert();
            w.master_slave().slave();
            w.left_right_dac_clock_swap().right_channel_dac_data_right();
            w.left_right_phase().data_when_daclrc_low();
            w.bit_length().bits_24();
            w.format().left_justified();
        }),
    );
    Timer::after_micros(10).await;

    // no clock division, normal mode
    write_wm8731_reg(
        i2c,
        WM8731::sampling(|w| {
            w.core_clock_divider_select().normal();
            w.base_oversampling_rate().normal_256();
            match fs {
                Fs::Fs8000 => {
                    w.sample_rate().adc_8();
                }
                Fs::Fs32000 => {
                    w.sample_rate().adc_32();
                }
                Fs::Fs44100 => {
                    w.sample_rate().adc_441();
                }
                Fs::Fs48000 => {
                    w.sample_rate().adc_48();
                }
                Fs::Fs88200 => {
                    w.sample_rate().adc_882();
                }
                Fs::Fs96000 => {
                    w.sample_rate().adc_96();
                }
            }
            w.usb_normal().normal();
        }),
    );
    Timer::after_micros(10).await;

    // set active
    write_wm8731_reg(i2c, WM8731::active().active());
    Timer::after_micros(10).await;

    //Note: WM8731's output not yet enabled.
}
fn write_wm8731_reg(i2c: &mut hal::i2c::I2c<'_, hal::mode::Blocking>, r: wm8731::Register) {
    const AD: u8 = 0x1a; // or 0x1b if CSB is high

    // WM8731 has 16 bits registers.
    // The first 7 bits are for the addresses, and the rest 9 bits are for the "value"s.
    // Let's pack wm8731::Register into 16 bits.
    let byte1: u8 = ((r.address << 1) & 0b1111_1110) | (((r.value >> 8) & 0b0000_0001) as u8);
    let byte2: u8 = (r.value & 0b1111_1111) as u8;
    unwrap!(i2c.blocking_write(AD, &[byte1, byte2]));
}
fn final_power_settings(w: &mut wm8731::power_down::PowerDown) {
    w.power_off().power_on();
    w.clock_output().power_off();
    w.oscillator().power_off();
    w.output().power_on();
    w.dac().power_on();
    w.adc().power_on();
    w.mic().power_off();
    w.line_input().power_on();
}

//================================================

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
