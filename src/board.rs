use crate::audio::{self, AudioBlockBuffers, AudioConfig, Interface};
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
    pub daisy_pins: DaisyPins,

    // board peripherals
    pub user_led: UserLed<'a>,
    pub interface: Interface<'a>,
    pub FMC: (),   //TODO
    pub SDRAM: (), // TODO
    pub daisy_usb: DaisyUsb,
}

pub struct DaisyPeripherals {
    pub daisy_pins: DaisyPins,
    pub led_user_pin: LedUserPin,
    pub wm8731_pin: WM8731Pins,
    pub audio_peripherals: audio::Peripherals,
    pub usb2_pins: USB2Pins,
    pub usb_otg_fs: USB_OTG_FS,
}

impl<'a> DaisyBoard<'a> {
    pub fn new(p: DaisyPeripherals, audio_config: AudioConfig) -> (Self, AudioBlockBuffers) {
        let usb_driver = crate::usb::init(p.usb_otg_fs, p.usb2_pins);
        let (interface, buffers) = Interface::new(p.wm8731_pin, p.audio_peripherals, audio_config);
        (
            Self {
                daisy_pins: p.daisy_pins,
                user_led: UserLed::new(p.led_user_pin),
                interface,
                FMC: (),
                SDRAM: (),
                daisy_usb: usb_driver,
            },
            buffers,
        )
    }
}
