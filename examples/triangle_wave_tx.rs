#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU8, Ordering};

use daisy_embassy::{
    audio::HALF_DMA_BUFFER_LENGTH,
    hal::{self, time::Hertz},
    new_daisy_boad,
};
use defmt::debug;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_time::Timer;
use hal::gpio::Pull;
use hal::{exti::ExtiInput, gpio::Input};
use {defmt_rtt as _, panic_probe as _};

#[derive(Clone, Copy)]
enum WaveFrequency {
    High,
    Middle,
    Low,
}

impl WaveFrequency {
    fn as_period(&self) -> u32 {
        match self {
            Self::High => 60,
            Self::Middle => 200,
            Self::Low => 400,
        }
    }
}

impl From<u8> for WaveFrequency {
    fn from(value: u8) -> Self {
        match value {
            0 => WaveFrequency::Low,
            1 => WaveFrequency::Middle,
            2 => WaveFrequency::High,
            _ => panic!(),
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    debug!("====program start====");
    let mut core = cortex_m::Peripherals::take().unwrap();
    core.SCB.enable_icache();
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
        config.rcc.pll2 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV8), // 100mhz
            divq: None,
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
    }

    let p = hal::init(config);
    let board = new_daisy_boad!(p);
    let (mut interface, (mut to_interface, mut from_interface)) = board
        .audio_peripherals
        .prepare_interface(Default::default())
        .await;
    let mute = Input::new(board.pins.d15, Pull::Up);
    let mut change_freq = ExtiInput::new(board.pins.d16, p.EXTI3, Pull::Up);
    let wave_freq = AtomicU8::new(0);

    let change_freq_fut = async {
        let mut local = 0;
        loop {
            change_freq.wait_for_low().await;
            local += 1;
            if local > 2 {
                local = 0;
            }
            wave_freq.store(local, Ordering::SeqCst);
            Timer::after_millis(30).await;
        }
    };
    let interface_fut = interface.start();

    let audio_callback_fut = async {
        let mut buf = [0; HALF_DMA_BUFFER_LENGTH];
        let mut smp_pos: u32 = 0;
        loop {
            //Receive buffer and discard all
            //This step is necessary for the audio callback to proceed.
            from_interface.receive().await;
            from_interface.receive_done();

            let period = WaveFrequency::from(wave_freq.load(Ordering::SeqCst)).as_period();
            for chunk in buf.chunks_mut(2) {
                let smp = f32_to_u24(make_triangle_wave(smp_pos % period, period));
                if mute.is_high() {
                    chunk[0] = smp;
                    chunk[1] = smp;
                } else {
                    //if user push mute button, do not send triangle wave
                    chunk[0] = 0;
                    chunk[1] = 0;
                }
                smp_pos = smp_pos.wrapping_add(1);
            }
            let tx = to_interface.send().await;
            tx.copy_from_slice(&buf);
            to_interface.send_done();
        }
    };
    join(change_freq_fut, join(interface_fut, audio_callback_fut)).await;
}

fn make_triangle_wave(pos: u32, period_smp: u32) -> f32 {
    assert!(pos <= period_smp);
    if pos <= (period_smp / 2) {
        pos as f32 * 4.0 / period_smp as f32 - 1.0
    } else {
        let pos = pos - period_smp / 2;
        pos as f32 * (-4.0) / period_smp as f32 + 1.0
    }
}

/// convert audio data from f32 to u24
#[inline(always)]
fn f32_to_u24(x: f32) -> u32 {
    //return (int16_t) __SSAT((int32_t) (x * 32767.f), 16);
    let x = x * 8_388_607.0;
    let x = x.clamp(-8_388_608.0, 8_388_607.0);
    (x as i32) as u32
}
