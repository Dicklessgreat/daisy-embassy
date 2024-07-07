use crate::pins::SdRamPins;
use cortex_m::peripheral::{MPU, SCB};
use embassy_stm32 as hal;
use hal::fmc::Fmc;
use hal::peripherals::FMC;
use is42s32160ge_75bli::Is42s32160ge75bli;
use stm32_fmc::Sdram;

pub struct SdRamBuilder {
    pub pins: SdRamPins,
    pub instance: FMC,
}

impl SdRamBuilder {
    pub fn build<'a>(self, mpu: &mut MPU, scb: &mut SCB) -> Sdram<Fmc<'a, FMC>, Is42s32160ge75bli> {
        // Configure MPU for external SDRAM
        // MPU config for SDRAM write-through
        let sdram_size = 64 * 1024 * 1024;
        // Refer to ARM®v7-M Architecture Reference Manual ARM DDI 0403
        // Version E.b Section B3.5
        const MEMFAULTENA: u32 = 1 << 16;
        unsafe {
            /* Make sure outstanding transfers are done */
            cortex_m::asm::dmb();

            scb.shcsr.modify(|r| r & !MEMFAULTENA);

            /* Disable the MPU and clear the control register*/
            mpu.ctrl.write(0);
        }

        const REGION_NUMBER0: u32 = 0x00;
        const REGION_BASE_ADDRESS: u32 = 0xD000_0000;

        const REGION_FULL_ACCESS: u32 = 0x03;
        const REGION_CACHEABLE: u32 = 0x01;
        const REGION_WRITE_BACK: u32 = 0x01;
        const REGION_ENABLE: u32 = 0x01;

        assert_eq!(
            sdram_size & (sdram_size - 1),
            0,
            "SDRAM memory region size must be a power of 2"
        );
        assert_eq!(
            sdram_size & 0x1F,
            0,
            "SDRAM memory region size must be 32 bytes or more"
        );
        fn log2minus1(sz: u32) -> u32 {
            for i in 5..=31 {
                if sz == (1 << i) {
                    return i - 1;
                }
            }
            panic!("Unknown SDRAM memory region size!");
        }

        // Configure region 0
        //
        // Cacheable, outer and inner write-back, no write allocate. So
        // reads are cached, but writes always write all the way to SDRAM
        unsafe {
            mpu.rnr.write(REGION_NUMBER0);
            mpu.rbar.write(REGION_BASE_ADDRESS);
            mpu.rasr.write(
                (REGION_FULL_ACCESS << 24)
                    | (REGION_CACHEABLE << 17)
                    | (REGION_WRITE_BACK << 16)
                    | (log2minus1(sdram_size as u32) << 1)
                    | REGION_ENABLE,
            );
        }

        const MPU_ENABLE: u32 = 0x01;
        const MPU_DEFAULT_MMAP_FOR_PRIVILEGED: u32 = 0x04;

        // Enable
        unsafe {
            mpu.ctrl
                .modify(|r| r | MPU_DEFAULT_MMAP_FOR_PRIVILEGED | MPU_ENABLE);

            scb.shcsr.modify(|r| r | MEMFAULTENA);

            // Ensure MPU settings take effect
            cortex_m::asm::dsb();
            cortex_m::asm::isb();
        }

        let Self { pins, instance } = self;
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
}

// Not yet implemented only boilerplate
//=====================is42s32160ge_75bli============================
#[allow(dead_code)]
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
