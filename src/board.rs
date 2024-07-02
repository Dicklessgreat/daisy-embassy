use crate::audio::{self, AudioPeripherals};
use crate::pins::*;
use crate::{led::UserLed, usb::DaisyUsb};
use embassy_stm32 as hal;
use hal::peripherals::USB_OTG_FS;
use hal::{bind_interrupts, i2c, peripherals, usb};

bind_interrupts!(pub struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

#[allow(non_snake_case)]
pub struct DaisyBoard<'a> {
    pub pins: DaisyPins,
    // board peripherals
    pub user_led: UserLed<'a>,
    pub audio_peripherals: AudioPeripherals,
    pub FMC: (),   //TODO
    pub SDRAM: (), // TODO
    pub daisy_usb: DaisyUsb,
    // on board "BOOT" button.
    pub boot: Boot,
}

pub struct DaisyPeripherals {
    pub daisy_pins: DaisyPins,
    pub led_user_pin: LedUserPin,
    pub audio_peripherals: audio::AudioPeripherals,
    pub usb2_pins: USB2Pins,
    pub usb_otg_fs: USB_OTG_FS,
    pub boot: Boot,
}

impl<'a> DaisyBoard<'a> {
    pub fn new(p: DaisyPeripherals) -> Self {
        let usb_driver = crate::usb::init(p.usb_otg_fs, p.usb2_pins);
        Self {
            pins: p.daisy_pins,
            user_led: UserLed::new(p.led_user_pin),
            audio_peripherals: p.audio_peripherals,
            FMC: (),
            SDRAM: (),
            daisy_usb: usb_driver,
            boot: p.boot,
        }
    }
}
