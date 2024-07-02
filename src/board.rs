use crate::audio::AudioPeripherals;
use crate::led::UserLed;
use crate::pins::*;
use crate::usb::UsbPeripherals;
use embassy_stm32 as hal;
use hal::{bind_interrupts, i2c, peripherals, usb};

bind_interrupts!(pub struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

pub struct DaisyBoard<'a> {
    pub pins: DaisyPins,
    // board peripherals
    pub user_led: UserLed<'a>,
    pub audio_peripherals: AudioPeripherals,
    pub fmc: (),   //TODO
    pub sdram: (), // TODO
    pub usb_peripherals: UsbPeripherals,
    // on board "BOOT" button.
    pub boot: Boot,
}
