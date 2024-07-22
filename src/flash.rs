use crate::pins::FlashPins;
use embassy_stm32::{
    self as hal,
    qspi::{
        enums::{AddressSize, DummyCycles, QspiWidth},
        TransferConfig,
    },
};
use hal::{
    mode::Blocking,
    peripherals::QUADSPI,
    qspi::{Config, Qspi},
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

pub struct FlashBuilder {
    pub pins: FlashPins,
    pub qspi: QUADSPI,
}

impl FlashBuilder {
    pub fn build<'a>(self) -> Flash<'a> {
        let mut config = Config::default();
        config.address_size = AddressSize::_32bit;
        let Self { pins, qspi } = self;
        let qspi = Qspi::new_blocking_bank1(
            qspi, pins.IO0, pins.IO1, pins.IO2, pins.IO3, pins.SCK, pins.CS, config,
        );
        Flash { qspi }
    }
}

pub struct Flash<'a> {
    qspi: Qspi<'a, QUADSPI, Blocking>,
}

impl<'a> Flash<'a> {
    pub fn read(&mut self, address: u32, buffer: &mut [u8]) {
        assert!(address <= MAX_ADDRESS);

        // Data must be queried by chunks of 32 (limitation of `read_extended`)
        for (i, chunk) in buffer.chunks_mut(32).enumerate() {
            let transaction = TransferConfig {
                iwidth: QspiWidth::SING,
                awidth: QspiWidth::QUAD,
                dwidth: QspiWidth::QUAD,
                instruction: FAST_READ_QUAD_IO_CMD,
                address: Some(address + i as u32 * 32),
                dummy: DummyCycles::_8,
            };
            self.qspi.blocking_read(chunk, transaction);
        }
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
                    iwidth: QspiWidth::SING,
                    awidth: QspiWidth::QUAD,
                    dwidth: QspiWidth::NONE,
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
                iwidth: QspiWidth::SING,
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
            iwidth: QspiWidth::SING,
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
                iwidth: QspiWidth::SING,
                awidth: QspiWidth::NONE,
                dwidth: QspiWidth::SING,
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
