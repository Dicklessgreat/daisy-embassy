use crate::pins::WM8731Pins;
use defmt::info;
use embassy_stm32 as hal;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    zerocopy_channel::{Channel, Receiver, Sender},
};
use embassy_time::Timer;
use grounded::uninit::GroundedArrayCell;
use hal::sai::BitOrder;
use hal::sai::ComplementFormat;
use hal::sai::FifoThreshold;
use hal::sai::FrameSyncOffset;
use hal::{
    peripherals,
    sai::{
        self, ClockStrobe, Config, DataSize, FrameSyncPolarity, MasterClockDivider, Mode, Sai,
        StereoMono, TxRx,
    },
    time::Hertz,
};
use static_cell::StaticCell;
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
pub type AudioBlockBuffers = (
    Sender<'static, NoopRawMutex, InterleavedBlock>,
    Receiver<'static, NoopRawMutex, InterleavedBlock>,
);

pub struct Interface<'a> {
    sai_tx_conf: sai::Config,
    sai_rx_conf: sai::Config,
    sai_tx: Sai<'a, peripherals::SAI1, u32>,
    sai_rx: Sai<'a, peripherals::SAI1, u32>,
    i2c: hal::i2c::I2c<'a, hal::mode::Blocking>,
    to_client: Sender<'static, NoopRawMutex, InterleavedBlock>,
    from_client: Receiver<'static, NoopRawMutex, InterleavedBlock>,
}

pub struct Peripherals {
    pub sai1: hal::peripherals::SAI1,
    pub i2c2: hal::peripherals::I2C2,
    pub dma1_ch1: hal::peripherals::DMA1_CH1,
    pub dma1_ch2: hal::peripherals::DMA1_CH2,
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

pub struct AudioConfig {
    tx_fs: Fs,
    rx_fs: Fs,
}

impl Default for AudioConfig {
    fn default() -> Self {
        AudioConfig {
            tx_fs: Fs::Fs48000,
            rx_fs: Fs::Fs48000,
        }
    }
}

impl<'a> Interface<'a> {
    pub async fn new(
        wm8731: WM8731Pins,
        p: Peripherals,
        audio_config: AudioConfig,
    ) -> (Self, AudioBlockBuffers) {
        let (sub_block_receiver, sub_block_transmitter) = hal::sai::split_subblocks(p.sai1);

        info!("set up i2c");
        let i2c_config = hal::i2c::Config::default();
        let mut i2c = embassy_stm32::i2c::I2c::new_blocking(
            p.i2c2, wm8731.SCL, wm8731.SDA, I2C_FS, i2c_config,
        );
        info!("set up WM8731");
        setup_wm8731(&mut i2c).await;

        info!("set up sai_tx");
        let sai_tx_conf = {
            let mut config = Config::default();
            config.mode = Mode::Slave;
            config.tx_rx = TxRx::Transmitter;
            config.stereo_mono = StereoMono::Stereo;
            config.data_size = DataSize::Data24;
            config.clock_strobe = ClockStrobe::Falling;
            config.frame_sync_polarity = FrameSyncPolarity::ActiveHigh;
            config.fifo_threshold = FifoThreshold::Empty;
            config.sync_output = false;
            config.bit_order = BitOrder::MsbFirst;
            config.complement_format = ComplementFormat::OnesComplement;
            config.frame_sync_offset = FrameSyncOffset::OnFirstBit;
            config.master_clock_divider = audio_config.tx_fs.into_clock_divider();
            config
        };
        let tx_buffer: &mut [u32] = unsafe {
            TX_BUFFER.initialize_all_copied(0);
            let (ptr, len) = TX_BUFFER.get_ptr_len();
            core::slice::from_raw_parts_mut(ptr, len)
        };
        let sai_tx = hal::sai::Sai::new_synchronous(
            sub_block_transmitter,
            wm8731.SD_B,
            p.dma1_ch1,
            tx_buffer,
            sai_tx_conf,
        );

        info!("set up sai_rx");
        let sai_rx_conf = {
            //copy tx configuration
            let mut config = sai_tx_conf;
            //fix rx only configuration
            config.mode = Mode::Master;
            config.tx_rx = TxRx::Receiver;
            config.clock_strobe = ClockStrobe::Rising;
            config.sync_output = true;
            config.master_clock_divider = audio_config.rx_fs.into_clock_divider();
            config
        };
        let rx_buffer: &mut [u32] = unsafe {
            RX_BUFFER.initialize_all_copied(0);
            let (ptr, len) = RX_BUFFER.get_ptr_len();
            core::slice::from_raw_parts_mut(ptr, len)
        };
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

        static TO_INTERFACE_BUF: StaticCell<[InterleavedBlock; 2]> = StaticCell::new();
        let to_interface_buf = TO_INTERFACE_BUF.init([[0; HALF_DMA_BUFFER_LENGTH]; 2]);
        static TO_INTERFACE: StaticCell<Channel<'_, NoopRawMutex, InterleavedBlock>> =
            StaticCell::new();
        let (client_to_if_tx, client_to_if_rx) =
            TO_INTERFACE.init(Channel::new(to_interface_buf)).split();
        static FROM_INTERFACE_BUF: StaticCell<[InterleavedBlock; 2]> = StaticCell::new();
        let from_interface_buf = FROM_INTERFACE_BUF.init([[0; HALF_DMA_BUFFER_LENGTH]; 2]);
        static FROM_INTERFACE: StaticCell<Channel<'_, NoopRawMutex, InterleavedBlock>> =
            StaticCell::new();
        let (if_to_client_tx, if_to_client_rx) = FROM_INTERFACE
            .init(Channel::new(from_interface_buf))
            .split();

        (
            Self {
                sai_rx_conf,
                sai_tx_conf,
                sai_rx,
                sai_tx,
                i2c,
                to_client: if_to_client_tx,
                from_client: client_to_if_rx,
            },
            (client_to_if_tx, if_to_client_rx),
        )
    }
    pub async fn start(&mut self) -> ! {
        info!("let's set up audio callback");
        info!("enable WM8731 output");
        write_wm8731_reg(
            &mut self.i2c,
            wm8731::WM8731::power_down(final_power_settings),
        );
        Timer::after_micros(10).await;

        info!("start SAI");
        self.sai_tx.start();
        self.sai_rx.start();

        info!("enter audio callback loop");
        loop {
            // Obtain a free buffer from the channel
            let buf = self.to_client.send().await;
            // and fill it with data
            self.sai_rx.read(buf).await.unwrap();
            //Notify the channel that the buffer is now ready to be received
            self.to_client.send_done();
            // await till client audio callback task has finished processing
            let buf = self.from_client.receive().await;
            self.sai_tx.write(buf).await.unwrap();
            self.from_client.receive_done();
        }
    }
    pub fn rx_config(&self) -> &sai::Config {
        &self.sai_rx_conf
    }
    pub fn tx_config(&self) -> &sai::Config {
        &self.sai_tx_conf
    }
}

//====================wm8731 register set up functions============================
async fn setup_wm8731<'a>(i2c: &mut hal::i2c::I2c<'a, hal::mode::Blocking>) {
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

    // nothing inverted, slave, 32-bits, MSB format
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

    // no clock division, normal mode, 48k
    write_wm8731_reg(
        i2c,
        WM8731::sampling(|w| {
            w.core_clock_divider_select().normal();
            w.base_oversampling_rate().normal_256();
            w.sample_rate().adc_48();
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
    i2c.blocking_write(AD, &[byte1, byte2]).unwrap();
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
