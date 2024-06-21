use embassy_stm32 as hal;
use hal::gpio::{self, Speed};
pub struct UserLed<'a>(gpio::Output<'a>);

impl<'a> UserLed<'a> {
    pub fn new(pin: hal::peripherals::PC7) -> Self {
        Self(gpio::Output::new(pin, gpio::Level::Low, Speed::Low))
    }
    pub fn on(&mut self) {
        self.0.set_high();
    }
    pub fn off(&mut self) {
        self.0.set_low();
    }
}
