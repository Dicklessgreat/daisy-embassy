#![no_std]
#![no_main]

use daisy_embassy::new_daisy_board;
use defmt::info;
use embassy_executor::Spawner;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = daisy_embassy::default_rcc();
    {
        use embassy_stm32::rcc::*;
        config.rcc.pll2 = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL216,
            divp: None,
            divq: None,
            divr: Some(PllDiv::DIV4), // 16mhz / 4 * 216 / 4 = 216Mhz,
            source: PllSource::HSE,
        });
        config.rcc.mux.quadspisel = embassy_stm32::pac::rcc::vals::Fmcsel::PLL2_R;
    }
    let p = embassy_stm32::init(config);
    let daisy_p = new_daisy_board!(p);
    // We will be using the first 8000 bytes of the flash.
    const ADDRESS: u32 = 0x00;
    const SIZE: usize = 8000;

    let mut flash = daisy_p.flash.build();

    info!("sr: {}", flash.read_sr());
    info!("id: {}", flash.read_id());
    info!("uuid: {}", flash.read_uuid());
    // Create an array of data to write.
    let mut data: [u8; SIZE] = [0; SIZE];
    for (i, x) in data.iter_mut().enumerate() {
        *x = (i % 256) as u8;
    }

    // Write it to the flash memory.
    info!("Writting to flash");
    flash.write_memory(ADDRESS, &data, false);

    // Read it back.
    info!("Reading from flash");
    let mut buffer: [u8; SIZE] = [0; SIZE];
    flash.read_memory(ADDRESS, &mut buffer, false);

    // Compare the read values with those written and lit the LED if they match.
    if data == buffer {
        info!("Everything went as expected");
    } else {
        info!("Read value does not match what was written");
    }

    // Sleep forever.
    loop {
        cortex_m::asm::nop();
    }
}
