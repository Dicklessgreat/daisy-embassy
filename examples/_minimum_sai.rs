//! this example does not belong to daisy_embassy,
//! but is to check proper settings of stm32h750's SAI and WM8731.
#![no_std]
#![no_main]
use defmt::info;
use defmt::warn;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32 as hal;
use embassy_stm32::sai::{ClockStrobe, Config, MasterClockDivider, Sai};
use embassy_time::Duration;
use embassy_time::Timer;
use grounded::uninit::GroundedArrayCell;
use hal::sai::DataSize;
use hal::sai::FrameSyncPolarity;
use hal::sai::Mode;
use hal::sai::StereoMono;
use hal::sai::TxRx;
use hal::time::Hertz;
use panic_probe as _;
// - global constants ---------------------------------------------------------

pub const BLOCK_LENGTH: usize = 32; // 32 samples
pub const HALF_DMA_BUFFER_LENGTH: usize = BLOCK_LENGTH * 2; //  2 channels
pub const DMA_BUFFER_LENGTH: usize = HALF_DMA_BUFFER_LENGTH * 2; //  2 half-blocks

//DMA buffer must be in special region. Refer https://embassy.dev/book/#_stm32_bdma_only_working_out_of_some_ram_regions
#[link_section = ".sram1_bss"]
static mut TX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();
#[link_section = ".sram1_bss"]
static mut RX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();

#[embassy_executor::task]
async fn execute(hal_config: hal::Config) {
    let p = hal::init(hal_config);
    let (sub_block_rx, sub_block_tx) = hal::sai::split_subblocks(p.SAI1);

    Timer::after(Duration::from_secs(2)).await;

    //setup codecs via I2C before init SAI.
    //Once SAI is initiated, the bus will be occupied by it.
    setup_codecs_from_i2c(p.I2C2, p.PH4, p.PB11).await;

    info!("Starting SAI");
    let (tx_config, mut sai_transmitter) = {
        let tx_config = {
            let mut config = Config::default();
            config.mode = Mode::Slave;
            config.tx_rx = TxRx::Transmitter;
            config.stereo_mono = StereoMono::Stereo;
            config.data_size = DataSize::Data32;
            config.clock_strobe = ClockStrobe::Falling;
            config.frame_sync_polarity = FrameSyncPolarity::ActiveHigh;
            let kernel_clock = hal::rcc::frequency::<hal::peripherals::SAI1>().0;
            info!("kernel clock:{}", kernel_clock);
            let mclk_div = (kernel_clock / (48000 * 256)) as u8;
            info!("master clock divider:{}", mclk_div);
            // config.mute_detection_counter = embassy_stm32::dma::word::U5(0);
            config.master_clock_divider = mclk_div_from_u8(mclk_div);
            // config.clock_strobe = ClockStrobe::Falling;
            // config.fifo_threshold = FifoThreshold::Empty;
            // config.complement_format = ComplementFormat::OnesComplement;
            // config.is_sync_output = true;
            config
        };

        let tx_buffer: &mut [u32] = unsafe {
            TX_BUFFER.initialize_all_copied(0);
            let (ptr, len) = TX_BUFFER.get_ptr_len();
            core::slice::from_raw_parts_mut(ptr, len)
        };
        (
            tx_config,
            Sai::new_synchronous(sub_block_tx, p.PE3, p.DMA1_CH0, tx_buffer, tx_config),
        )
    };

    let mut sai_receiver = {
        let rx_config = {
            let mut config = tx_config;
            config.mode = Mode::Master;
            config.tx_rx = TxRx::Receiver;
            config.clock_strobe = ClockStrobe::Rising;
            // config.is_sync_output = false;
            config
        };
        let rx_buffer: &mut [u32] = unsafe {
            RX_BUFFER.initialize_all_copied(0);
            let (ptr, len) = RX_BUFFER.get_ptr_len();
            core::slice::from_raw_parts_mut(ptr, len)
        };

        Sai::new_asynchronous_with_mclk(
            sub_block_rx,
            p.PE5,
            p.PE6,
            p.PE4,
            p.PE2,
            p.DMA1_CH1,
            rx_buffer,
            rx_config,
        )
    };

    let mut smp_pos = 0;
    let mut signal = [0; HALF_DMA_BUFFER_LENGTH];

    sai_receiver.start();
    sai_transmitter.start();

    let mut rx_signal = [0u32; HALF_DMA_BUFFER_LENGTH];
    // const NUM_ITERATE: usize = 10;
    // buffers to store received audio samples.
    // let mut rx_signal_buf = [[0u32; HALF_DMA_BUFFER_LENGTH]; NUM_ITERATE];

    info!("enter audio loop");
    loop {
        //fill the buffer
        for chunk in signal.chunks_mut(2) {
            let smp = make_triangle_wave(smp_pos % DUR);
            chunk[0] = smp;
            chunk[1] = smp;
            smp_pos += 1;
        }

        match sai_transmitter.write(&signal).await {
            Ok(_) => {}
            Err(e) => {
                warn!("Error writing to SAI: {:?}", e);
            }
        }
        match sai_receiver.read(&mut rx_signal).await {
            Ok(_) => {}
            Err(e) => {
                warn!("Error reading from SAI: {:?}", e);
            }
        }

        // for (i, value) in rx_signal.iter().enumerate() {
        //     if *value != signal[i] {
        //         info!("[{}]: {} != {}", i, value, signal[i]); //we don't want to see this
        //         break;
        //     }
        // }
        // *buf = rx_signal;
    }

    // for buf in rx_signal_buf {
    //     info!("{}", buf);
    //     // printing each rx_signal_buf costs too much.
    //     // let's prevent print-out buffer from overflowing.
    //     Timer::after(Duration::from_secs(1)).await;
    // }

    // info!("finished execution");
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = hal::Config::default();
    {
        use hal::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8),
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(16),
            mode: HseMode::Oscillator,
        });
        //default as PLL1_Q?
        // use hal::pac::rcc::vals::Saisel;
        // config.rcc.mux.sai1sel = Saisel::PLL1_Q;
    }
    spawner.spawn(execute(config)).unwrap();
}

async fn setup_codecs_from_i2c(
    i2c2: hal::peripherals::I2C2,
    ph4: hal::peripherals::PH4,
    pb11: hal::peripherals::PB11,
) {
    use wm8731::{power_down, WM8731};
    info!("setup codecs from I2C");
    let i2c_config = hal::i2c::Config::default();
    let mut i2c =
        embassy_stm32::i2c::I2c::new_blocking(i2c2, ph4, pb11, Hertz(100_000), i2c_config);

    fn final_power_settings(w: &mut power_down::PowerDown) {
        w.power_off().power_on();
        w.clock_output().power_off();
        w.oscillator().power_off();
        w.output().power_on();
        w.dac().power_on();
        w.adc().power_on();
        w.mic().power_off();
        w.line_input().power_on();
    }
    fn write(i2c: &mut embassy_stm32::i2c::I2c<'static, hal::mode::Blocking>, r: wm8731::Register) {
        const AD: u8 = 0x1a; // or 0x1b if CSB is high
        let byte1: u8 = ((r.address << 1) & 0b1111_1110) | ((r.value as u8 >> 7) & 0b0000_0001u8);
        i2c.blocking_write(AD, &[byte1, r.value.try_into().unwrap()])
            .unwrap();
    }
    Timer::after_micros(10).await;

    // reset
    write(&mut i2c, WM8731::reset());
    Timer::after_micros(10).await;

    // wakeup
    write(
        &mut i2c,
        WM8731::power_down(|w| {
            final_power_settings(w);
            //output off during initialization
            w.output().power_off();
        }),
    );
    Timer::after_micros(10).await;

    // disable input mute, set to 0dB gain
    write(
        &mut i2c,
        WM8731::left_line_in(|w| {
            w.both().disable();
            w.mute().disable();
            w.volume().nearest_dB(0);
        }),
    );
    Timer::after_micros(10).await;

    // sidetone off; DAC selected; bypass off; line input selected; mic muted; mic boost off
    write(
        &mut i2c,
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
    write(
        &mut i2c,
        WM8731::digital_audio_path(|w| {
            w.dac_mut().disable();
            w.deemphasis().frequency_48();
        }),
    );
    Timer::after_micros(10).await;

    // nothing inverted, slave, 32-bits, MSB format
    write(
        &mut i2c,
        WM8731::digital_audio_interface_format(|w| {
            w.bit_clock_invert().no_invert();
            w.master_slave().slave();
            w.left_right_dac_clock_swap().right_channel_dac_data_right();
            w.left_right_phase().data_when_daclrc_low();
            w.bit_length().bits_32();
            w.format().left_justified();
        }),
    );
    Timer::after_micros(10).await;

    // no clock division, normal mode, 48k
    write(
        &mut i2c,
        WM8731::sampling(|w| {
            w.core_clock_divider_select().normal();
            w.base_oversampling_rate().normal_256();
            w.sample_rate().adc_48();
            w.usb_normal().normal();
        }),
    );
    Timer::after_micros(10).await;

    // set active
    write(&mut i2c, WM8731::active().active());
    Timer::after_micros(10).await;

    // enable output
    write(&mut i2c, WM8731::power_down(final_power_settings));
    Timer::after_micros(10).await;
}

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

const DUR: u32 = 120;
const fn make_triangle_wave(pos: u32) -> u32 {
    assert!(pos <= DUR);
    let half = u32::MAX / 2;
    if pos <= (DUR / 4) {
        half + (pos * (half / DUR * 4))
    } else if (DUR / 4) < pos && pos <= (DUR / 4 * 3) {
        let pos = pos - DUR / 4;
        u32::MAX - (pos * (u32::MAX / DUR * 2))
    } else {
        let pos = pos - DUR / 4 * 3;
        (half / DUR * 4) * pos
    }
}
