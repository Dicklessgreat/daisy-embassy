//! This example demonstrates recording incoming audio to SD card.
//! By setting the D16 pin low, you can record audio from the SAI input.
//! It triggers to record the audio of next 10 seconds, and automatically stop recording.
//! After it stops recording, it dumps the recorded as WAV file to SD card.
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use daisy_embassy::{audio::HALF_DMA_BUFFER_LENGTH, hal, new_daisy_board, sdram::SDRAM_SIZE};
use defmt::{debug, error, info, unwrap};
use embassy_executor::Spawner;
use embassy_futures::join::join4;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{self, Pull},
    spi::Spi,
    time::Hertz,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex};
use embassy_time::{Delay, Timer};
use embedded_sdmmc::{self, SdCard, VolumeIdx, VolumeManager};
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
    let sdram: Mutex<CriticalSectionRawMutex, _> =
        Mutex::new(board.sdram.build(&mut c.MPU, &mut c.SCB));
    let event: Channel<CriticalSectionRawMutex, RecordingHasFinished, 1> = Channel::new();
    let event_tx = event.sender();

    let audio_callback_fut = async {
        // Block Length
        const BL: usize = HALF_DMA_BUFFER_LENGTH;
        //record point
        let mut rp = 0;
        loop {
            let rx = from_interface.receive().await;
            // if triggered record, record incoming buffer till the loop buffer is full
            if RECORD.load(Ordering::SeqCst) {
                // The only time SDRAM is used elsewhere is when flushing recorded sound,
                // and flushing only happens when the recording has been finished.
                // When it fails to acquire lock and some audio samples are missed,
                // let's take that it is not a "failure".
                if let Ok(mut sdram) = sdram.try_lock() {
                    let mut delay = Delay;
                    let loop_buffer = unsafe {
                        // Initialise controller and SDRAM
                        let ram_ptr: *mut u32 = sdram.init(&mut delay) as *mut _;

                        // Convert raw pointer to slice
                        core::slice::from_raw_parts_mut(
                            ram_ptr,
                            SDRAM_SIZE / core::mem::size_of::<u32>(),
                        )
                    };

                    let remain = BL.min(RECORD_LENGTH - rp);
                    loop_buffer[rp..(rp + remain)].copy_from_slice(rx);
                    rp += BL;
                    if rp >= RECORD_LENGTH {
                        rp = 0;
                        RECORD.store(false, Ordering::SeqCst);
                        // do not block. discard event if it fails
                        if event_tx.try_send(RecordingHasFinished).is_err() {
                            error!("Recording finish event queue is full!!");
                        }
                        info!("finished recording");
                    }
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
            Timer::after_secs(10).await;
        }
    };
    let event_rx = event.receiver();
    let mut spi_config = embassy_stm32::spi::Config::default();
    // frequency must be low during card initialization
    spi_config.frequency = Hertz::khz(400);
    let spi = Spi::new(
        p.SPI1,
        board.pins.d8,
        board.pins.d10,
        board.pins.d9,
        p.DMA1_CH0,
        p.DMA1_CH3,
        spi_config,
    );
    let spi = defmt::unwrap!(embedded_hal_bus::spi::ExclusiveDevice::new(
        spi,
        gpio::Output::new(board.pins.d7, gpio::Level::High, gpio::Speed::High),
        Delay,
    ));
    let sdcard = SdCard::new(spi, Delay);
    let mut vmg0 = VolumeManager::new(sdcard, DummyTimeSource);
    // initialize sd card by printing out sdcard size
    info!("sdcard size: {}", defmt::unwrap!(vmg0.device().num_bytes()));
    spi_config.frequency = Hertz::mhz(15);
    vmg0.device()
        .spi(|spi| defmt::unwrap!(spi.bus_mut().set_config(&spi_config)));
    let mut volume = defmt::unwrap!(vmg0.open_volume(VolumeIdx(0)));
    let mut dir = defmt::unwrap!(volume.open_root_dir());
    let mut file = unwrap!(dir.open_file_in_dir(
        "recorded.wav",
        embedded_sdmmc::Mode::ReadWriteCreateOrTruncate,
    ));
    // dump recorded to SD card
    let dump_fut = async {
        loop {
            event_rx.receive().await;
            info!("flush the recorded to SD card");
            let mut sdram = sdram.lock().await;
            let mut delay = Delay;
            let loop_buffer = unsafe {
                // Initialise controller and SDRAM
                let ram_ptr: *mut u32 = sdram.init(&mut delay) as *mut _;

                // Convert raw pointer to slice
                core::slice::from_raw_parts_mut(ram_ptr, SDRAM_SIZE / core::mem::size_of::<u32>())
            };
            unwrap!(file.write(&wav_header()));
            for chunk in loop_buffer[..RECORD_LENGTH].chunks(1 << 15) {
                let mut tmp = [0; 1 << 16];
                for (i, smp) in chunk.iter().enumerate() {
                    let bytes = ((smp >> 8) as u16).to_le_bytes();
                    tmp[i * 2] = bytes[0];
                    tmp[i * 2 + 1] = bytes[1];
                }
                unwrap!(file.write(&tmp));
            }
            unwrap!(file.flush());
            info!("finish flushing to sd card!!");
        }
    };
    join4(interface.start(), audio_callback_fut, record_fut, dump_fut).await;
}

/// 16bit, 48KHz canonical wav container header
fn wav_header() -> [u8; 44] {
    let mut result = [0; 44];
    //Note chunk sizes are little endian
    //RIFF chunk
    result[0] = b'R';
    result[1] = b'I';
    result[2] = b'F';
    result[3] = b'F';
    //RIFF chunk size(1920000 + 36 == 0x001d4c24)
    result[4] = 0x24;
    result[5] = 0x4c;
    result[6] = 0x1d;
    result[7] = 0x00;
    //WAVE identifier
    result[8] = b'W';
    result[9] = b'A';
    result[10] = b'V';
    result[11] = b'E';
    //fmt chunk
    result[12] = b'f';
    result[13] = b'm';
    result[14] = b't';
    result[15] = b' ';
    //fmt size
    result[16] = 0x10;
    result[17] = 0x00;
    result[18] = 0x00;
    result[19] = 0x00;
    //fmt content
    //format(1 == pcm)
    result[20] = 1;
    result[21] = 0;
    //num channels
    result[22] = 2;
    result[23] = 0;
    //sampling rate(48000 == 0x0000bb80)
    result[24] = 0x80;
    result[25] = 0xbb;
    result[26] = 0;
    result[27] = 0;
    //bytes per sec
    result[28] = 0x00;
    result[29] = 0xee;
    result[30] = 0x02;
    result[31] = 0x00;
    //I forgot what this is
    result[32] = 0x04;
    result[33] = 0x00;
    // bits per sample(0x0010 == 16bit)
    result[34] = 0x10;
    result[35] = 0x00;
    //data chunk
    result[36] = b'd';
    result[37] = b'a';
    result[38] = b't';
    result[39] = b'a';
    //data size(960000 * 16 / 8 = 1_920_000 == 0x001d4c00)
    result[40] = 0x00;
    result[41] = 0x4c;
    result[42] = 0x1d;
    result[43] = 0x00;
    result
}

struct DummyTimeSource;
impl embedded_sdmmc::TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}
