#![no_std]
#![no_main]

use daisy_embassy::new_daisy_board;
use defmt::info;
use embassy_executor::Spawner;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = daisy_embassy::default_rcc();
    let p = embassy_stm32::init(config);
    let daisy_p = new_daisy_board!(p);

    // We will be using the first 8000 bytes of the flash.
    const ADDRESS: u32 = 0x00;
    const SIZE: usize = 8000;

    let mut flash = daisy_p.flash.build();

    info!("uuid: {}", flash.read_uuid());
    // Create an array of data to write.
    let mut data: [u8; SIZE] = [0; SIZE];
    for (i, x) in data.iter_mut().enumerate() {
        *x = (i % 256) as u8;
    }
    info!("Write buffer: {:?}", data[0..32]);

    // Write it to the flash memory.
    info!("Writting to flash");
    flash.write(ADDRESS, &data);

    // Read it back.
    info!("Reading from flash");
    let mut buffer: [u8; SIZE] = [0; SIZE];
    flash.read(ADDRESS, &mut buffer);
    info!("Read buffer: {:?}", buffer[0..32]);

    if data == buffer {
        info!("Everything went as expected");
    } else {
        info!("Read value does not match what was written");
    }
}
