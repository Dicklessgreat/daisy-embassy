use crate::hal;
use crate::pins::FlashPins;
use embassy_stm32::qspi::enums::{AddressSize, ChipSelectHighTime, FIFOThresholdLevel, MemorySize};
use hal::{
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

pub struct FlashBuilder {
    pub pins: FlashPins,
    pub qspi: QUADSPI,
}

impl FlashBuilder {
    pub fn build<'a>(self) -> Flash<'a> {
        let config = hal::qspi::Config {
            memory_size: MemorySize::_8MiB,
            address_size: AddressSize::_24bit,
            prescaler: 1,
            cs_high_time: ChipSelectHighTime::_2Cycle,
            fifo_threshold: FIFOThresholdLevel::_1Bytes,
        };
        let Self { pins, qspi } = self;
        let qspi = Qspi::new_blocking_bank1(
            qspi, pins.IO0, pins.IO1, pins.IO2, pins.IO3, pins.SCK, pins.CS, config,
        );
        let mut result = Flash { qspi };
        result.enable_qpi_mode();
        result.reset_status_register();
        result.reset_read_register();
        result
    }
}
const MAX_ADDRESS: u32 = 0x7FFFFF;

pub struct Flash<'a> {
    qspi: Qspi<'a, QUADSPI, Blocking>,
}

impl Flash<'_> {
    pub fn read(&mut self, address: u32, buffer: &mut [u8]) {
        assert!(address <= MAX_ADDRESS);

        let transaction = TransferConfig {
            iwidth: QspiWidth::QUAD,
            awidth: QspiWidth::QUAD,
            dwidth: QspiWidth::QUAD,
            instruction: FAST_READ_QUAD_IO_CMD,
            address: Some(address),
            dummy: DummyCycles::_6,
        };
        self.qspi.blocking_read(buffer, transaction);
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

        //WRITE_CMD(or PP) allows to write up to 256 bytes, which is as much as PAGE_SIZE.
        //Let's divide the data into chunks of page size to write to flash
        loop {
            // Calculate number of bytes between address and end of the page.
            let page_remainder = PAGE_SIZE - (address & (PAGE_SIZE - 1));
            let size = page_remainder.min(length) as usize;
            self.enable_write();
            let transaction = TransferConfig {
                iwidth: QspiWidth::QUAD,
                awidth: QspiWidth::QUAD,
                dwidth: QspiWidth::QUAD,
                instruction: WRITE_CMD,
                address: Some(address),
                dummy: DummyCycles::_0,
            };

            self.qspi
                .blocking_write(&data[start_cursor..start_cursor + size], transaction);
            self.wait_for_write();
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

            self.qspi.command(transaction);
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
        self.qspi.command(transaction);
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
        self.qspi.command(transaction);
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
        self.qspi.command(transaction);
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
        self.qspi.command(transaction);
        self.wait_for_write();
    }
}
