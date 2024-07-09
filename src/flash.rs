use crate::pins::FlashPins;
use embassy_stm32 as hal;
use hal::{
    mode::Blocking,
    peripherals::QUADSPI,
    qspi::{Config, Qspi},
};

pub struct FlashBuilder {
    pub pins: FlashPins,
    pub qspi: QUADSPI,
}

impl FlashBuilder {
    pub fn build<'a>(self) -> Flash<'a> {
        let config = Config::default();
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
