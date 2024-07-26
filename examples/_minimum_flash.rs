#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{
    self as hal,
    qspi::{
        enums::{AddressSize, ChipSelectHighTime, FIFOThresholdLevel, MemorySize},
        Qspi,
    },
    time::Hertz,
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

mod flash {
    use embassy_stm32::{
        mode::Blocking,
        peripherals::QUADSPI,
        qspi::{
            enums::{DummyCycles, QspiWidth},
            Qspi, TransferConfig,
        },
    };

    // Commands from IS25LP064 datasheet.
    const WRITE_STATUS_REGISTRY_CMD: u8 = 0x01; // WRSR
    const WRITE_CMD: u8 = 0x02; // PP
    const READ_STATUS_REGISTRY_CMD: u8 = 0x05; // RDSR
    const WRITE_ENABLE_CMD: u8 = 0x06; // WREN
    const ENTER_QPI_MODE_CMD: u8 = 0x35; // QPIEN
    const SET_READ_PARAMETERS_CMD: u8 = 0xC0; // SRP
    const SECTOR_ERASE_CMD: u8 = 0xD7; // SER
    const FAST_READ_QUAD_IO_CMD: u8 = 0xEB; // FRQIO

    // Memory array specifications as defined in the datasheet.
    const SECTOR_SIZE: u32 = 4096;
    const PAGE_SIZE: u32 = 256;
    const MAX_ADDRESS: u32 = 0x7FFFFF;

    pub struct Flash<'a> {
        qspi: Qspi<'a, QUADSPI, Blocking>,
    }

    impl<'a> Flash<'a> {
        pub fn new(qspi: Qspi<'a, QUADSPI, Blocking>) -> Self {
            let mut flash = Self { qspi };
            flash.enable_qpi_mode();
            flash.reset_status_register();
            flash.reset_read_register();
            flash
        }

        pub fn read(&mut self, address: u32, buffer: &mut [u8]) {
            assert!(address <= MAX_ADDRESS);

            // Data must be queried by chunks of 32 (limitation of `read_extended`)
            for (i, chunk) in buffer.chunks_mut(32).enumerate() {
                let transaction = TransferConfig {
                    iwidth: QspiWidth::QUAD,
                    awidth: QspiWidth::QUAD,
                    dwidth: QspiWidth::QUAD,
                    instruction: FAST_READ_QUAD_IO_CMD,
                    address: Some(address + i as u32 * 32),
                    dummy: DummyCycles::_6,
                };
                self.qspi.blocking_read(chunk, transaction);
            }
        }

        pub fn read_uuid(&mut self) -> [u8; 16] {
            let mut buffer = [0; 16];
            let transaction: TransferConfig = TransferConfig {
                iwidth: QspiWidth::QUAD,
                awidth: QspiWidth::QUAD,
                dwidth: QspiWidth::QUAD,
                instruction: 0x4B,
                address: Some(0x00),
                dummy: DummyCycles::_6,
            };
            self.qspi.blocking_read(&mut buffer, transaction);
            buffer
        }

        pub fn write(&mut self, mut address: u32, data: &[u8]) {
            assert!(address <= MAX_ADDRESS);
            assert!(!data.is_empty());
            self.erase(address, data.len() as u32);

            let mut length = data.len() as u32;
            let mut start_cursor = 0;

            loop {
                // Calculate number of bytes between address and end of the page.
                let page_remainder = PAGE_SIZE - (address & (PAGE_SIZE - 1));
                // Write data to the page in chunks of 32 (limitation of `write_extended`).
                let size = page_remainder.min(length) as usize;
                for (i, chunk) in data[start_cursor..start_cursor + size]
                    .chunks(32)
                    .enumerate()
                {
                    self.enable_write();
                    let transaction = TransferConfig {
                        iwidth: QspiWidth::QUAD,
                        awidth: QspiWidth::QUAD,
                        dwidth: QspiWidth::QUAD,
                        instruction: WRITE_CMD,
                        address: Some(address + i as u32 * 32),
                        dummy: DummyCycles::_0,
                    };

                    self.qspi.blocking_write(chunk, transaction);
                    self.wait_for_write();
                }
                start_cursor += size;

                // Stop if this was the last needed page.
                if length <= page_remainder {
                    break;
                }
                length -= page_remainder;

                // Jump to the next page.
                address += page_remainder;
                address %= MAX_ADDRESS;
            }
        }

        pub fn erase(&mut self, mut address: u32, mut length: u32) {
            assert!(address <= MAX_ADDRESS);
            assert!(length > 0);

            loop {
                // Erase the sector.
                self.enable_write();
                let transaction = TransferConfig {
                    iwidth: QspiWidth::QUAD,
                    awidth: QspiWidth::QUAD,
                    dwidth: QspiWidth::NONE,
                    instruction: SECTOR_ERASE_CMD,
                    address: Some(address),
                    dummy: DummyCycles::_0,
                };

                self.qspi.blocking_write(&[], transaction);
                self.wait_for_write();

                // Calculate number of bytes between address and end of the sector.
                let sector_remainder = SECTOR_SIZE - (address & (SECTOR_SIZE - 1));

                // Stop if this was the last affected sector.
                if length <= sector_remainder {
                    break;
                }
                length -= sector_remainder;

                // Jump to the next sector.
                address += sector_remainder;
                address %= MAX_ADDRESS;
            }
        }

        fn enable_write(&mut self) {
            let transaction = TransferConfig {
                iwidth: QspiWidth::QUAD,
                awidth: QspiWidth::NONE,
                dwidth: QspiWidth::NONE,
                instruction: WRITE_ENABLE_CMD,
                address: None,
                dummy: DummyCycles::_0,
            };
            self.qspi.blocking_write(&[], transaction);
        }

        fn wait_for_write(&mut self) {
            loop {
                let mut status: [u8; 1] = [0xFF; 1];
                let transaction = TransferConfig {
                    iwidth: QspiWidth::QUAD,
                    awidth: QspiWidth::NONE,
                    dwidth: QspiWidth::QUAD,
                    instruction: READ_STATUS_REGISTRY_CMD,
                    address: None,
                    dummy: DummyCycles::_0,
                };
                self.qspi.blocking_read(&mut status, transaction);

                if status[0] & 0x01 == 0 {
                    break;
                }
            }
        }

        /// Reset status registers into driver's defaults. This makes sure that the
        /// peripheral is configured as expected.
        fn reset_status_register(&mut self) {
            self.enable_write();
            let transaction = TransferConfig {
                iwidth: QspiWidth::QUAD,
                awidth: QspiWidth::QUAD,
                dwidth: QspiWidth::NONE,
                instruction: WRITE_STATUS_REGISTRY_CMD,
                address: Some(0b0000_0010),
                dummy: DummyCycles::_0,
            };
            self.qspi.blocking_write(&[], transaction);
            self.wait_for_write();
        }

        /// Reset read registers into driver's defaults. This makes sure that the
        /// peripheral is configured as expected.
        fn reset_read_register(&mut self) {
            self.enable_write();
            let transaction = TransferConfig {
                iwidth: QspiWidth::QUAD,
                awidth: QspiWidth::QUAD,
                dwidth: QspiWidth::NONE,
                instruction: SET_READ_PARAMETERS_CMD,
                address: Some(0b1111_1000),
                dummy: DummyCycles::_0,
            };
            self.qspi.blocking_write(&[], transaction);
            self.wait_for_write();
        }

        fn enable_qpi_mode(&mut self) {
            self.enable_write();
            let transaction = TransferConfig {
                iwidth: QspiWidth::SING,
                awidth: QspiWidth::NONE,
                dwidth: QspiWidth::NONE,
                instruction: ENTER_QPI_MODE_CMD,
                address: None,
                dummy: DummyCycles::_0,
            };
            self.qspi.blocking_write(&[], transaction);
            self.wait_for_write();
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = hal::Config::default();
    use hal::rcc::*;
    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV4,
        // mul: PllMul::MUL200, // 400MHz
        mul: PllMul::MUL240, // 480MHz
        divp: Some(PllDiv::DIV2),
        divq: Some(PllDiv::DIV5),
        divr: Some(PllDiv::DIV2),
    });
    config.rcc.pll3 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV6,
        mul: PllMul::MUL295,
        divp: Some(PllDiv::DIV16),
        divq: Some(PllDiv::DIV4),
        divr: Some(PllDiv::DIV32),
    });
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.mux.sai1sel = hal::pac::rcc::vals::Saisel::PLL3_P;

    config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
    config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
    config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
    config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
    config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
    config.rcc.hse = Some(Hse {
        freq: Hertz::mhz(16),
        mode: HseMode::Oscillator,
    });

    let p = hal::init(config);

    let mut config = hal::qspi::Config::default();
    config.address_size = AddressSize::_24bit;
    config.memory_size = MemorySize::_8MiB;
    config.prescaler = 1;
    config.cs_high_time = ChipSelectHighTime::_2Cycle;
    config.fifo_threshold = FIFOThresholdLevel::_1Bytes;

    let qspi =
        Qspi::new_blocking_bank1(p.QUADSPI, p.PF8, p.PF9, p.PF7, p.PF6, p.PF10, p.PG6, config);
    let mut flash = flash::Flash::new(qspi);

    let uuid = flash.read_uuid();
    info!("Flash UUID: {:x}", uuid);

    const ADDRESS: u32 = 0x00;
    const SIZE: usize = 8000;

    // Write some data to flash
    let mut write_buf: [u8; SIZE] = [0; SIZE];
    for (i, x) in write_buf.iter_mut().enumerate() {
        *x = (i % 256) as u8;
    }
    flash.write(ADDRESS, &write_buf);

    // Read it back from flash
    let mut read_buf: [u8; SIZE] = [0; SIZE];
    flash.read(ADDRESS, &mut read_buf);

    // Assert read data == written data
    defmt::assert!(read_buf == write_buf);
    info!("Assertions succeeded.");

    loop {
        Timer::after_millis(1000).await;
    }
}
