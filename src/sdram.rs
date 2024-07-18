use crate::pins::SdRamPins;
use cortex_m::peripheral::{MPU, SCB};
use embassy_stm32 as hal;
use hal::fmc::Fmc;
use hal::peripherals::FMC;
use stm32_fmc::devices::as4c16m32msa_6::As4c16m32msa;
use stm32_fmc::Sdram;

pub const SDRAM_SIZE: usize = 64 * 1024 * 1024;
pub struct SdRamBuilder {
    pub pins: SdRamPins,
    pub instance: FMC,
}

impl SdRamBuilder {
    pub fn build<'a>(self, mpu: &mut MPU, scb: &mut SCB) -> Sdram<Fmc<'a, FMC>, As4c16m32msa> {
        // Configure MPU for external SDRAM
        // MPU config for SDRAM write-through
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
            SDRAM_SIZE & (SDRAM_SIZE - 1),
            0,
            "SDRAM memory region size must be a power of 2"
        );
        assert_eq!(
            SDRAM_SIZE & 0x1F,
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
                    | (log2minus1(SDRAM_SIZE as u32) << 1)
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
        Fmc::sdram_a13bits_d32bits_4banks_bank1(
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
            pins.gg2,
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
            As4c16m32msa {},
        )
    }
}
