use embassy_stm32 as hal;
use hal::{peripherals::USB_OTG_FS, usb::Driver};

use crate::pins::USB2Pins;

pub type DaisyUsb = Driver<'static, USB_OTG_FS>;

pub struct UsbPeripherals {
    pub usb_otg_fs: USB_OTG_FS,
    pub pins: USB2Pins,
}
