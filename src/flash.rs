use crate::pins::FlashPins;
use defmt::{debug, info};
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
// const WRITE_STATUS_REGISTRY_CMD: u8 = 0x01; // WRSR
// const WRITE_CMD: u8 = 0x02; // PP
// const READ_STATUS_REGISTRY_CMD: u8 = 0x05; // RDSR
// const WRITE_ENABLE_CMD: u8 = 0x06; // WREN
const ENTER_QPI_MODE_CMD: u8 = 0x35; // QPIEN
const SET_READ_PARAMETERS_CMD: u8 = 0xC0; // SRP

// const SECTOR_ERASE_CMD: u8 = 0xD7; // SER

// const FAST_READ_QUAD_IO_CMD: u8 = 0xEB; // FRQIO

const CMD_ENABLE_RESET: u8 = 0x66;
const CMD_RESET: u8 = 0x99;
const CMD_WRITE_ENABLE: u8 = 0x06; // WREN
const CMD_READ_ID: u8 = 0xab;
const CMD_READ_UUID: u8 = 0x4b;
const CMD_QUAD_READ: u8 = 0xeb;
const CMD_SECTOR_ERASE: u8 = 0xd7;
const CMD_BLOCK_ERASE_32K: u8 = 0x52;
const CMD_BLOCK_ERASE_64K: u8 = 0xd8;
const CMD_CHIP_ERASE: u8 = 0xc7;
const CMD_READ_SR: u8 = 0x05;
const CMD_WRITE_SR: u8 = 0x01;
const CMD_QUAD_WRITE_PG: u8 = 0xeb;

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
        let config = Config {
            address_size: AddressSize::_32bit,
            ..Default::default()
        };
        let Self { pins, qspi } = self;
        let qspi = Qspi::new_blocking_bank1(
            qspi, pins.IO0, pins.IO1, pins.IO2, pins.IO3, pins.SCK, pins.CS, config,
        );
        let mut result = Flash { qspi };
        result.reset_memory();
        // result.enable_quad();
        result.enable_qpi_mode();
        result.write_sr(0b0000_0010);
        // result.reset_status_register();
        result.reset_read_register();
        result
    }
}

pub struct Flash<'a> {
    qspi: Qspi<'a, QUADSPI, Blocking>,
}

impl<'a> Flash<'a> {
    // fn enable_quad(&mut self) {
    //     let cr = self.read_cr();
    //     self.write_cr(cr | 0x02);
    // }

    fn exec_command(&mut self, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::NONE,
            instruction: cmd,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.command(transaction);
    }

    pub fn reset_memory(&mut self) {
        self.exec_command(CMD_ENABLE_RESET);
        self.exec_command(CMD_RESET);
        self.wait_write_finish();
    }

    pub fn enable_write(&mut self) {
        self.exec_command(CMD_WRITE_ENABLE);
    }

    pub fn read_id(&mut self) -> [u8; 2] {
        let mut buffer = [0; 2];
        let transaction: TransferConfig = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: CMD_READ_ID,
            address: None,
            dummy: DummyCycles::_24,
        };
        self.qspi.blocking_read(&mut buffer, transaction);
        buffer
    }

    pub fn read_uuid(&mut self) -> [u8; 16] {
        let mut buffer = [0; 16];
        for i in 0..16 {
            let transaction: TransferConfig = TransferConfig {
                iwidth: QspiWidth::SING,
                awidth: QspiWidth::SING,
                dwidth: QspiWidth::SING,
                instruction: CMD_READ_UUID,
                address: Some(i as u32),
                dummy: DummyCycles::_8,
            };
            self.qspi
                .blocking_read(&mut buffer[i..(i + 1)], transaction);
        }
        buffer
    }

    pub fn read_memory(&mut self, addr: u32, buffer: &mut [u8], use_dma: bool) {
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::SING,
            dwidth: QspiWidth::QUAD,
            instruction: CMD_QUAD_READ,
            address: Some(addr),
            dummy: DummyCycles::_8,
        };
        if use_dma {
            todo!()
            // self.qspi.blocking_read_dma(buffer, transaction);
        } else {
            self.qspi.blocking_read(buffer, transaction);
        }
    }

    fn wait_write_finish(&mut self) {
        while (self.read_sr() & 0x01) != 0 {}
    }

    fn perform_erase(&mut self, addr: u32, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::SING,
            dwidth: QspiWidth::NONE,
            instruction: cmd,
            address: Some(addr),
            dummy: DummyCycles::_0,
        };
        self.enable_write();
        self.qspi.command(transaction);
        self.wait_write_finish();
    }

    pub fn erase_sector(&mut self, addr: u32) {
        self.perform_erase(addr, CMD_SECTOR_ERASE);
    }

    pub fn erase_block_32k(&mut self, addr: u32) {
        self.perform_erase(addr, CMD_BLOCK_ERASE_32K);
    }

    pub fn erase_block_64k(&mut self, addr: u32) {
        self.perform_erase(addr, CMD_BLOCK_ERASE_64K);
    }

    pub fn erase_chip(&mut self) {
        self.exec_command(CMD_CHIP_ERASE);
    }

    fn write_page(&mut self, addr: u32, buffer: &[u8], len: usize, use_dma: bool) {
        assert!(
            (len as u32 + (addr & 0x000000ff)) <= PAGE_SIZE,
            "write_page(): page write length exceeds page boundary (len = {}, addr = {:X}",
            len,
            addr
        );

        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::SING,
            dwidth: QspiWidth::QUAD,
            instruction: CMD_QUAD_WRITE_PG,
            address: Some(addr),
            dummy: DummyCycles::_0,
        };
        self.enable_write();
        if use_dma {
            // self.qspi.blocking_write_dma(buffer, transaction);
            todo!()
        } else {
            self.qspi.blocking_write(buffer, transaction);
        }
        self.wait_write_finish();
    }

    pub fn write_memory(&mut self, addr: u32, buffer: &[u8], use_dma: bool) {
        let mut left = buffer.len();
        let mut place = addr;
        let mut chunk_start = 0;

        while left > 0 {
            let max_chunk_size = (PAGE_SIZE - (place & 0x000000ff)) as usize;
            let chunk_size = if left >= max_chunk_size {
                max_chunk_size
            } else {
                left
            };
            let chunk = &buffer[chunk_start..(chunk_start + chunk_size)];
            self.write_page(place, chunk, chunk_size, use_dma);
            place += chunk_size as u32;
            left -= chunk_size;
            chunk_start += chunk_size;
        }
    }

    fn read_register(&mut self, cmd: u8) -> u8 {
        let mut buffer = [0; 1];
        let transaction: TransferConfig = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: cmd,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_read(&mut buffer, transaction);
        buffer[0]
    }

    fn write_register(&mut self, cmd: u8, value: u8) {
        self.enable_write();
        let buffer = [value; 1];
        let transaction: TransferConfig = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: cmd,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_write(&buffer, transaction);
        self.wait_write_finish();
    }

    pub fn read_sr(&mut self) -> u8 {
        self.read_register(CMD_READ_SR)
    }

    // pub fn read_cr(&mut self) -> u8 {
    //     self.read_register(CMD_READ_CR)
    // }

    pub fn write_sr(&mut self, value: u8) {
        self.write_register(CMD_WRITE_SR, value);
    }

    // pub fn write_cr(&mut self, value: u8) {
    //     self.write_register(CMD_WRITE_CR, value);
    // }
    // pub fn read(&mut self, address: u32, buffer: &mut [u8]) {
    //     assert!(address <= MAX_ADDRESS);

    //     // Data must be queried by chunks of 32 (limitation of `read_extended`)
    //     for (i, chunk) in buffer.chunks_mut(32).enumerate() {
    //         let transaction = TransferConfig {
    //             iwidth: QspiWidth::SING,
    //             awidth: QspiWidth::QUAD,
    //             dwidth: QspiWidth::QUAD,
    //             instruction: FAST_READ_QUAD_IO_CMD,
    //             address: Some(address + i as u32 * 32),
    //             dummy: DummyCycles::_8,
    //         };
    //         self.qspi.blocking_read(chunk, transaction);
    //     }
    // }
    // pub fn write(&mut self, mut address: u32, data: &[u8]) {
    //     assert!(address <= MAX_ADDRESS);
    //     assert!(!data.is_empty());

    //     debug!("erase before write");
    //     self.erase(address, data.len() as u32);

    //     debug!("let's write");
    //     self.enable_write();
    //     let transaction = TransferConfig {
    //         iwidth: QspiWidth::SING,
    //         awidth: QspiWidth::QUAD,
    //         dwidth: QspiWidth::NONE,
    //         instruction: WRITE_CMD,
    //         address: Some(address),
    //         dummy: DummyCycles::_0,
    //     };

    //     self.qspi.blocking_write(data, transaction);
    //     self.wait_for_write();

    //     // let mut length = data.len() as u32;
    //     // let mut start_cursor = 0;

    //     // loop {
    //     //     // Calculate number of bytes between address and end of the page.
    //     //     let page_remainder = PAGE_SIZE - (address & (PAGE_SIZE - 1));

    //     //     // Write data to the page in chunks of 32 (limitation of `write_extended`).
    //     //     let size = page_remainder.min(length) as usize;
    //     //     for (i, chunk) in data[start_cursor..start_cursor + size]
    //     //         .chunks(32)
    //     //         .enumerate()
    //     //     {
    //     //         self.enable_write();
    //     //         let transaction = TransferConfig {
    //     //             iwidth: QspiWidth::SING,
    //     //             awidth: QspiWidth::QUAD,
    //     //             dwidth: QspiWidth::NONE,
    //     //             instruction: WRITE_CMD,
    //     //             address: Some(address + i as u32 * 32),
    //     //             dummy: DummyCycles::_0,
    //     //         };

    //     //         self.qspi.blocking_write(chunk, transaction);
    //     //         self.wait_for_write();
    //     //     }
    //     //     start_cursor += size;

    //     //     // Stop if this was the last needed page.
    //     //     if length <= page_remainder {
    //     //         break;
    //     //     }
    //     //     length -= page_remainder;

    //     //     // Jump to the next page.
    //     //     address += page_remainder;
    //     //     address %= MAX_ADDRESS;
    //     // }
    // }

    // pub fn erase(&mut self, mut address: u32, mut length: u32) {
    //     assert!(address <= MAX_ADDRESS);
    //     assert!(length > 0);

    //     loop {
    //         // Erase the sector.
    //         self.enable_write();
    //         let transaction = TransferConfig {
    //             iwidth: QspiWidth::SING,
    //             awidth: QspiWidth::SING,
    //             dwidth: QspiWidth::NONE,
    //             instruction: SECTOR_ERASE_CMD,
    //             address: Some(address),
    //             dummy: DummyCycles::_0,
    //         };

    //         self.qspi.command(transaction);

    //         self.wait_for_write();

    //         // Calculate number of bytes between address and end of the sector.
    //         let sector_remainder = SECTOR_SIZE - (address & (SECTOR_SIZE - 1));

    //         // Stop if this was the last affected sector.
    //         if length <= sector_remainder {
    //             break;
    //         }
    //         length -= sector_remainder;

    //         // Jump to the next sector.
    //         address += sector_remainder;
    //         address %= MAX_ADDRESS;
    //     }
    // }
    // fn enable_write(&mut self) {
    //     let transaction = TransferConfig {
    //         iwidth: QspiWidth::SING,
    //         awidth: QspiWidth::NONE,
    //         dwidth: QspiWidth::NONE,
    //         instruction: WRITE_ENABLE_CMD,
    //         address: None,
    //         dummy: DummyCycles::_0,
    //     };

    //     self.qspi.command(transaction);
    // }
    // fn wait_for_write(&mut self) {
    //     loop {
    //         let mut status: [u8; 1] = [0xFF; 1];
    //         let transaction = TransferConfig {
    //             iwidth: QspiWidth::SING,
    //             awidth: QspiWidth::NONE,
    //             dwidth: QspiWidth::SING,
    //             instruction: READ_STATUS_REGISTRY_CMD,
    //             address: None,
    //             dummy: DummyCycles::_0,
    //         };

    //         self.qspi.blocking_read(&mut status, transaction);

    //         if status[0] & 0x01 == 0 {
    //             break;
    //         }
    //     }
    // }

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

        self.wait_write_finish();
    }
    // fn reset_status_register(&mut self) {
    //     self.enable_write();

    //     let transaction = TransferConfig {
    //         iwidth: QspiWidth::SING,
    //         awidth: QspiWidth::NONE,
    //         dwidth: QspiWidth::NONE,
    //         instruction: WRITE_STATUS_REGISTRY_CMD,
    //         address: None,
    //         dummy: DummyCycles::_0,
    //     };

    //     // In daisy crate with stm32h7xx-hal, this data was written as an "address",
    //     // but if you give the transaction an address and
    //     // call command() in the same way,
    //     // you get stuck in an infinite loop.
    //     // https://github.com/zlosynth/daisy/blob/c827f2c088412ed195800ded68218dc0375ed573/src/flash.rs#L223
    //     // https://github.com/stm32-rs/stm32h7xx-hal/blob/5166da8a5485d51e60a42d3a564d1edae0c8e164/src/xspi/mod.rs#L795
    //     // also see IS25LP032 datasheet p.44 "8.16 WRITE STATUS REGISTER OPERATION (WRSR, 01h)"
    //     self.qspi.blocking_write(&[0b0000_0010], transaction);

    //     self.wait_for_write();
    // }

    fn reset_read_register(&mut self) {
        self.enable_write();

        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::NONE,
            instruction: SET_READ_PARAMETERS_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };

        // In daisy crate with stm32h7xx-hal, this data was written as an "address",
        // but if you give the transaction an address and
        // call command() in the same way,
        // you get stuck in an infinite loop.
        // https://github.com/zlosynth/daisy/blob/c827f2c088412ed195800ded68218dc0375ed573/src/flash.rs#L240        // https://github.com/stm32-rs/stm32h7xx-hal/blob/5166da8a5485d51e60a42d3a564d1edae0c8e164/src/xspi/mod.rs#L795
        // also see IS25LP032 datasheet p.51 "8.23 SET READ PARAMETERS OPERATION (SRP, C0h)"
        self.qspi.blocking_write(&[0b1111_1000], transaction);

        self.wait_write_finish();
    }
}
