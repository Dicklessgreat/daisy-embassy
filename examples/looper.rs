//! This example demonstrates the loop playback of audio mapped on SDRAM.
//! By setting the D16 pin low, you can record audio from the SAI input.
//! It overwrites the audio buffer from start to end and automatically stops recording after 10 seconds.
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use daisy_embassy::{audio::HALF_DMA_BUFFER_LENGTH, hal, new_daisy_board, sdram::SDRAM_SIZE};
use defmt::{debug, info};
use embassy_executor::{InterruptExecutor, Spawner};
use embassy_futures::join::join;
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::{InterruptExt, Priority};
use embassy_stm32::{exti::ExtiInput, gpio::Pull};
use embassy_time::Delay;
use {defmt_rtt as _, panic_probe as _};

//take 48000(Hz) * 10(Sec) * 2(stereo)
const LOOPER_LENGTH: usize = 960_000;
const SILENCE: u32 = u32::MAX / 2;
static RECORD: AtomicBool = AtomicBool::new(false);
static AUDIO_EXECUTOR: InterruptExecutor = InterruptExecutor::new();

#[interrupt]
unsafe fn SAI1() {
    AUDIO_EXECUTOR.on_interrupt()
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    debug!("====program start====");
    let config = daisy_embassy::default_rcc();
    let p = hal::init(config);
    let mut c = cortex_m::Peripherals::take().unwrap();
    let board = new_daisy_board!(p);
    let (mut interface, (mut to_interface, mut from_interface)) = board
        .audio_peripherals
        .prepare_interface(Default::default())
        .await;
    let mut sdram = board.sdram.build(&mut c.MPU, &mut c.SCB);
    let mut delay = Delay;
    let ram_slice = unsafe {
        // Initialise controller and SDRAM
        let ram_ptr: *mut u32 = sdram.init(&mut delay) as *mut _;

        // Convert raw pointer to slice
        core::slice::from_raw_parts_mut(ram_ptr, SDRAM_SIZE / core::mem::size_of::<u32>())
    };
    let loop_buffer = &mut ram_slice[..LOOPER_LENGTH];
    //clear loop_buffer
    for smp in loop_buffer.iter_mut() {
        *smp = SILENCE;
    }

    let audio_callback_fut = async move {
        // Block Length
        const BL: usize = HALF_DMA_BUFFER_LENGTH;
        //record point
        let mut rp = 0;
        //playback point
        let mut pp = 0;
        loop {
            let rx = from_interface.receive().await;
            // if triggered record, record incoming buffer till the loop buffer is full
            if RECORD.load(Ordering::SeqCst) {
                let remain = BL.min(LOOPER_LENGTH - rp);
                loop_buffer[rp..(rp + remain)].copy_from_slice(rx);
                rp += BL;
                if rp >= LOOPER_LENGTH {
                    rp = 0;
                    RECORD.store(false, Ordering::SeqCst);
                    info!("finished recording");
                }
            }
            from_interface.receive_done();

            let tx = to_interface.send().await;
            let remain = BL.min(LOOPER_LENGTH - pp);
            let frac = BL - remain;
            tx.copy_from_slice(&loop_buffer[pp..(pp + remain)]);
            if frac > 0 {
                tx[remain..BL].copy_from_slice(&loop_buffer[0..frac]);
            }
            pp += BL;
            if pp >= LOOPER_LENGTH {
                pp -= LOOPER_LENGTH;
                info!("loop!!");
            }
            to_interface.send_done();
        }
    };
    let mut record_pin = ExtiInput::new(board.pins.d16, p.EXTI3, Pull::Up);
    let record_fut = async {
        loop {
            record_pin.wait_for_low().await;
            info!("record!!");
            RECORD.store(true, Ordering::SeqCst);
        }
    };

    interrupt::SAI1.set_priority(Priority::P6);
    let spawner = AUDIO_EXECUTOR.start(interrupt::SAI1);
    defmt::unwrap!(spawner.spawn(join(interface.start(), audio_callback_fut)));
    join(audio_callback_fut, record_fut).await;
}
