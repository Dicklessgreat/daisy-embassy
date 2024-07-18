//! from embassy/examples/stm32h7/src/bin/fmc.rs

#![no_std]
#![no_main]

use daisy_embassy::new_daisy_board;
use daisy_embassy::sdram::SDRAM_SIZE;
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Delay, Timer};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let daisy_p = new_daisy_board!(p);
    let mut core = cortex_m::Peripherals::take().unwrap();
    let mut sdram = daisy_p.sdram.build(&mut core.MPU, &mut core.SCB);

    let mut delay = Delay;

    let ram_slice = unsafe {
        // Initialise controller and SDRAM
        let ram_ptr: *mut u32 = sdram.init(&mut delay) as *mut _;

        // Convert raw pointer to slice
        core::slice::from_raw_parts_mut(ram_ptr, SDRAM_SIZE / core::mem::size_of::<u32>())
    };

    info!("RAM contents before writing: {:x}", ram_slice[..10]);

    ram_slice[0] = 1;
    ram_slice[1] = 2;
    ram_slice[2] = 3;
    ram_slice[3] = 4;

    info!("RAM contents after writing: {:x}", ram_slice[..10]);

    defmt::assert_eq!(ram_slice[0], 1);
    defmt::assert_eq!(ram_slice[1], 2);
    defmt::assert_eq!(ram_slice[2], 3);
    defmt::assert_eq!(ram_slice[3], 4);

    info!("Assertions succeeded.");

    loop {
        Timer::after_millis(1000).await;
    }
}
