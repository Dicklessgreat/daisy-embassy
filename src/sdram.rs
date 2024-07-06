use embassy_stm32 as hal;
use hal::fmc::Fmc;
use hal::peripherals;
use is42s32160ge_75bli::Is42s32160ge75bli;
use stm32_fmc::Sdram;

pub struct SdRamPins {
    dd0: peripherals::PD0,
    dd1: peripherals::PD1,
    dd8: peripherals::PD8,
    dd9: peripherals::PD9,
    dd10: peripherals::PD10,
    dd14: peripherals::PD14,
    dd15: peripherals::PD15,
    ee0: peripherals::PE0,
    ee1: peripherals::PE1,
    ee7: peripherals::PE7,
    ee8: peripherals::PE8,
    ee9: peripherals::PE9,
    ee10: peripherals::PE10,
    ee11: peripherals::PE11,
    ee12: peripherals::PE12,
    ee13: peripherals::PE13,
    ee14: peripherals::PE14,
    ee15: peripherals::PE15,
    ff0: peripherals::PF0,
    ff1: peripherals::PF1,
    ff2: peripherals::PF2,
    ff3: peripherals::PF3,
    ff4: peripherals::PF4,
    ff5: peripherals::PF5,
    ff11: peripherals::PF11,
    ff12: peripherals::PF12,
    ff13: peripherals::PF13,
    ff14: peripherals::PF14,
    ff15: peripherals::PF15,
    gg0: peripherals::PG0,
    gg1: peripherals::PG1,
    gg2: peripherals::PG2,
    gg4: peripherals::PG4,
    gg5: peripherals::PG5,
    gg8: peripherals::PG8,
    gg15: peripherals::PG15,
    hh2: peripherals::PH2,
    hh3: peripherals::PH3,
    hh5: peripherals::PH5,
    hh8: peripherals::PH8,
    hh9: peripherals::PH9,
    hh10: peripherals::PH10,
    hh11: peripherals::PH11,
    hh12: peripherals::PH12,
    hh13: peripherals::PH13,
    hh14: peripherals::PH14,
    hh15: peripherals::PH15,
    ii0: peripherals::PI0,
    ii1: peripherals::PI1,
    ii2: peripherals::PI2,
    ii3: peripherals::PI3,
    ii4: peripherals::PI4,
    ii5: peripherals::PI5,
    ii6: peripherals::PI6,
    ii7: peripherals::PI7,
    ii9: peripherals::PI9,
    ii10: peripherals::PI10,
}

fn init<'a>(
    pins: SdRamPins,
    instance: peripherals::FMC,
) -> Sdram<Fmc<'a, peripherals::FMC>, Is42s32160ge75bli> {
    Fmc::sdram_a12bits_d32bits_4banks_bank1(
        instance,
        // A0-A12
        pins.ff0,
        pins.ff1,
        pins.ff2,
        pins.ff3,
        pins.ff4,
        pins.ff5,
        pins.ff12,
        pins.ff13,
        pins.ff14,
        pins.ff15,
        pins.gg0,
        pins.gg1,
        // is42s32160ge_75bli has "A12" pin, but not yet implemented
        // pins.gg2,

        // BA0-BA1
        pins.gg4,
        pins.gg5,
        // D0-D31
        pins.dd14,
        pins.dd15,
        pins.dd0,
        pins.dd1,
        pins.ee7,
        pins.ee8,
        pins.ee9,
        pins.ee10,
        pins.ee11,
        pins.ee12,
        pins.ee13,
        pins.ee14,
        pins.ee15,
        pins.dd8,
        pins.dd9,
        pins.dd10,
        pins.hh8,
        pins.hh9,
        pins.hh10,
        pins.hh11,
        pins.hh12,
        pins.hh13,
        pins.hh14,
        pins.hh15,
        pins.ii0,
        pins.ii1,
        pins.ii2,
        pins.ii3,
        pins.ii6,
        pins.ii7,
        pins.ii9,
        pins.ii10,
        // NBL0 - NBL3
        pins.ee0,
        pins.ee1,
        pins.ii4,
        pins.ii5,
        pins.hh2,  // SDCKE0
        pins.gg8,  // SDCLK
        pins.gg15, // SDNCAS
        pins.hh3,  // SDNE0
        pins.ff11, // SDRAS
        pins.hh5,  // SDNWE
        Is42s32160ge75bli {},
    )
}

// Not yet implemented only boilerplate
//=====================is42s32160ge_75bli============================
mod is42s32160ge_75bli {

    use stm32_fmc::{SdramChip, SdramConfiguration, SdramTiming};

    const BURST_LENGTH_1: u16 = 0x0000;
    const BURST_LENGTH_2: u16 = 0x0001;
    const BURST_LENGTH_4: u16 = 0x0002;
    const BURST_LENGTH_8: u16 = 0x0004;
    const BURST_TYPE_SEQUENTIAL: u16 = 0x0000;
    const BURST_TYPE_INTERLEAVED: u16 = 0x0008;
    const CAS_LATENCY_2: u16 = 0x0020;
    const CAS_LATENCY_3: u16 = 0x0030;
    const OPERATING_MODE_STANDARD: u16 = 0x0000;
    const WRITEBURST_MODE_PROGRAMMED: u16 = 0x0000;
    const WRITEBURST_MODE_SINGLE: u16 = 0x0200;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Is42s32160ge75bli {}

    impl SdramChip for Is42s32160ge75bli {
        /// Value of the mode register
        const MODE_REGISTER: u16 = BURST_LENGTH_1
            | BURST_TYPE_SEQUENTIAL
            | CAS_LATENCY_3
            | OPERATING_MODE_STANDARD
            | WRITEBURST_MODE_SINGLE;

        /// Timing Parameters
        const TIMING: SdramTiming = SdramTiming {
            startup_delay_ns: 100_000,    // 100 µs
            max_sd_clock_hz: 100_000_000, // 100 MHz
            refresh_period_ns: 15_625,    // 64ms / (4096 rows) = 15625ns
            mode_register_to_active: 2,   // tMRD = 2 cycles
            exit_self_refresh: 7,         // tXSR = 70ns
            active_to_precharge: 4,       // tRAS = 42ns
            row_cycle: 7,                 // tRC = 70ns
            row_precharge: 2,             // tRP = 18ns
            row_to_column: 2,             // tRCD = 18ns
        };

        /// SDRAM controller configuration
        const CONFIG: SdramConfiguration = SdramConfiguration {
            column_bits: 9,
            row_bits: 12,
            memory_data_width: 32, // 32-bit
            internal_banks: 4,     // 4 internal banks
            cas_latency: 3,        // CAS latency = 3
            write_protection: false,
            read_burst: true,
            read_pipe_delay_cycles: 0,
        };
    }
}
