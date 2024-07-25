//! this example does not belong to daisy_embassy,
//! but is to check proper settings of stm32h750's QSPI with IS25LP064.

#![no_std]
#![no_main]
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32 as hal;
use embassy_stm32::qspi::enums::AddressSize;
use embassy_stm32::qspi::enums::ChipSelectHighTime;
use embassy_stm32::qspi::enums::DummyCycles;
use embassy_stm32::qspi::enums::FIFOThresholdLevel;
use embassy_stm32::qspi::enums::MemorySize;
use embassy_stm32::qspi::enums::QspiWidth;
use embassy_stm32::qspi::Qspi;
use embassy_stm32::qspi::TransferConfig;
use panic_probe as _;

// WRSR datasheet p.44
const WRITE_STATUS_REGISTER_OPERATION: u8 = 0x06;
// WREN datasheet p.41
const WRITE_ENABLE_OPERATION: u8 = 0x06;
// RDSR datasheet p.43
const READ_STATUS_REGISTER_OPERATION: u8 = 0x05;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = hal::Config::default();
    {
        use hal::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: hal::time::Hertz::mhz(16),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL200,
            divp: Some(PllDiv::DIV2), //system clock. 16 / 4 * 200 / 2 = 400MHz
            divq: Some(PllDiv::DIV17), //QSPI clock. 16 / 4 * 200 / 17 = 47.05...MHz
            divr: None,
        });
        config.rcc.mux.quadspisel = embassy_stm32::pac::rcc::vals::Fmcsel::PLL1_Q;
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
    }
    let p = hal::init(config);
    let mut qspi = Qspi::new_blocking_bank1(
        //https://github.com/electro-smith/libDaisy/blob/3dda55e9ed55a2f8b6bc4fa6aa2c7ae134c317ab/src/per/qspi.c#L695
        p.QUADSPI,
        p.PF8,
        p.PF9,
        p.PF7,
        p.PF6,
        p.PB2,
        p.PG6,
        hal::qspi::Config {
            memory_size: MemorySize::_8MiB,
            address_size: AddressSize::_24bit,
            // from https://github.com/electro-smith/libDaisy/blob/3dda55e9ed55a2f8b6bc4fa6aa2c7ae134c317ab/src/per/qspi.c#L75
            prescaler: 1,
            cs_high_time: ChipSelectHighTime::_2Cycle,
            fifo_threshold: FIFOThresholdLevel::_1Bytes,
        },
    );

    let expected_sr = 0b0000_0010;
    //enable write
    let mut buffer = [0; 2];
    let transaction = TransferConfig {
        iwidth: QspiWidth::SING,
        awidth: QspiWidth::NONE,
        dwidth: QspiWidth::NONE,
        instruction: WRITE_ENABLE_OPERATION,
        address: None,
        dummy: DummyCycles::_0,
    };
    qspi.blocking_read(&mut buffer, transaction);

    // write to status register
    let transaction = TransferConfig {
        iwidth: QspiWidth::SING,
        awidth: QspiWidth::NONE,
        dwidth: QspiWidth::NONE,
        instruction: WRITE_STATUS_REGISTER_OPERATION,
        address: None,
        dummy: DummyCycles::_0,
    };
    qspi.blocking_write(&[expected_sr], transaction);

    // read from status register
    // wait till the write operation has finished
    let sr = loop {
        let mut status: [u8; 1] = [0xFF; 1];
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: READ_STATUS_REGISTER_OPERATION,
            address: None,
            dummy: DummyCycles::_0,
        };
        qspi.blocking_read(&mut status, transaction);
        // When it finishes the write operation, LSB in status register becomes 0
        if status[0] & 0x01 == 0 {
            break status[0];
        }
    };
    info!("status register {:b}", sr);
    assert_eq!(sr, expected_sr);
}
