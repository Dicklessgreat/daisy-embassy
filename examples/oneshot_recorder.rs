//! This example demonstrates recording incoming audio to SD card.
//! By setting the D16 pin low, you can record audio from the SAI input.
//! It triggers to record the audio of next 10 seconds, and automatically stop recording.
//! After it stops recording, it dumps the recorded as WAV file to SD card.
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use daisy_embassy::{audio::HALF_DMA_BUFFER_LENGTH, hal, new_daisy_board, sdram::SDRAM_SIZE};
use defmt::{debug, error, info};
use embassy_executor::Spawner;
use embassy_futures::join::join4;
use embassy_stm32::{exti::ExtiInput, gpio::Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Delay;
use {defmt_rtt as _, panic_probe as _};

struct RecordingHasFinished;
//record 48000(Hz) * 10(Sec) * 2(stereo)
const RECORD_LENGTH: usize = 960_000;
const SILENCE: u32 = u32::MAX / 2;
static RECORD: AtomicBool = AtomicBool::new(false);

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
    let loop_buffer = &mut ram_slice[..RECORD_LENGTH];
    //clear loop_buffer
    for smp in loop_buffer.iter_mut() {
        *smp = SILENCE;
    }
    let event: Channel<CriticalSectionRawMutex, RecordingHasFinished, 1> = Channel::new();
    let event_tx = event.sender();

    let audio_callback_fut = async move {
        // Block Length
        const BL: usize = HALF_DMA_BUFFER_LENGTH;
        //record point
        let mut rp = 0;
        loop {
            let rx = from_interface.receive().await;
            // if triggered record, record incoming buffer till the loop buffer is full
            if RECORD.load(Ordering::SeqCst) {
                let remain = BL.min(RECORD_LENGTH - rp);
                loop_buffer[rp..(rp + remain)].copy_from_slice(rx);
                rp += BL;
                if rp >= RECORD_LENGTH {
                    rp = 0;
                    RECORD.store(false, Ordering::SeqCst);
                    // do not block. discard event if it fails to
                    if event_tx.try_send(RecordingHasFinished).is_err() {
                        error!("Recording finish event queue is full!!");
                    }
                    info!("finished recording");
                }
            }
            from_interface.receive_done();

            //clear DA buffer
            to_interface
                .send()
                .await
                .iter_mut()
                .for_each(|smp| *smp = SILENCE);
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
    let event_rx = event.receiver();
    // dump recorded to SD card
    let dump_fut = async {
        loop {
            event_rx.receive().await;
            // todo!();
        }
    };
    join4(interface.start(), audio_callback_fut, record_fut, dump_fut).await;
}
