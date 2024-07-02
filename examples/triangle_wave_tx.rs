#![no_std]
#![no_main]

use daisy_embassy::{
    audio::HALF_DMA_BUFFER_LENGTH,
    hal::{self, time::Hertz},
    new_daisy_p,
    pins::{DaisyPins, USB2Pins, WM8731Pins},
    DaisyBoard,
};
use defmt::debug;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
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
    fn next(&mut self) {
        match self {
            Self::High => *self = Self::Middle,
            Self::Middle => *self = Self::Low,
            Self::Low => *self = Self::High,
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
        config.rcc.mux.adcsel = mux::Adcsel::PLL2_P;
    }

    let p = hal::init(config);
    let daisy_p = new_daisy_p!(p);
    let (board, (mut to_interface, mut from_interface)) =
        DaisyBoard::new(daisy_p, Default::default()).await;
    let mut interface = board.interface;
    let mute = Input::new(board.pins.d15, Pull::Up);
    let mut change_freq = ExtiInput::new(board.pins.d16, p.EXTI3, Pull::Up);
    let freq_queue: Channel<CriticalSectionRawMutex, (), 4> = Channel::new();
    let freq_sender = freq_queue.sender();
    let freq_receiver = freq_queue.receiver();

    let change_freq_fut = async {
        loop {
            change_freq.wait_for_low().await;
            freq_sender.send(()).await;
            Timer::after_millis(30).await;
        }
    };
    let interface_fut = async { interface.start().await };

    let audio_callback_fut = async {
        let mut buf = [0; HALF_DMA_BUFFER_LENGTH];
        let mut smp_pos = 0;
        let mut freq = WaveFrequency::Middle;
        loop {
            //Receive buffer and discard all
            //This step is necessary for the audio callback to proceed.
            from_interface.receive().await;
            from_interface.receive_done();

            if freq_receiver.try_receive().is_ok() {
                freq.next();
            }
            let period = freq.as_period();
            for chunk in buf.chunks_mut(2) {
                let smp = make_triangle_wave(smp_pos % period, period);
                if mute.is_high() {
                    chunk[0] = smp;
                    chunk[1] = smp;
                } else {
                    //if user push mute button, do not send triangle wave
                    chunk[0] = 0;
                    chunk[1] = 0;
                }
                smp_pos = smp_pos.wrapping_add_signed(1);
            }
            let tx = to_interface.send().await;
            tx.copy_from_slice(&buf);
            to_interface.send_done();
        }
    };
    join(change_freq_fut, join(interface_fut, audio_callback_fut)).await;
}

const fn make_triangle_wave(pos: u32, period_smp: u32) -> u32 {
    assert!(pos <= period_smp);
    let half = u32::MAX / 2;
    if pos <= (period_smp / 4) {
        half + (pos * (half / period_smp * 4))
    } else if (period_smp / 4) < pos && pos <= (period_smp / 4 * 3) {
        let pos = pos - period_smp / 4;
        u32::MAX - (pos * (u32::MAX / period_smp * 2))
    } else {
        let pos = pos - period_smp / 4 * 3;
        (half / period_smp * 4) * pos
    }
}
