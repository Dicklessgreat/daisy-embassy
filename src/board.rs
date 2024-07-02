use crate::audio::AudioPeripherals;
use crate::led::UserLed;
use crate::pins::*;
use crate::usb::UsbPeripherals;
use embassy_stm32 as hal;
use hal::{bind_interrupts, peripherals, usb};

bind_interrupts!(pub struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
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
