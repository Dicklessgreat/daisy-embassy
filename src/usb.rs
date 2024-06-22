use embassy_stm32 as hal;
use hal::{
    peripherals::USB_OTG_FS,
    usb::{Config, Driver},
};

use crate::pins::USB2Pins;

pub type DaisyUsb<'a> = Driver<'a, USB_OTG_FS>;

pub fn init<'a>(
    usb_otg_fs: USB_OTG_FS,
    pins: USB2Pins,
    ep_out_buffer: &'a mut [u8; 256],
) -> DaisyUsb<'a> {
    let mut config = Config::default();
    // Do not enable vbus_detection. This is a safe default that works in all boards.
    // However, if your USB device is self-powered (can stay powered on if USB is unplugged), you need
    // to enable vbus_detection to comply with the USB spec. If you enable it, the board
    // has to support it or USB won't work at all. See docs on `vbus_detection` for details.
    config.vbus_detection = false;
    Driver::new_fs(usb_otg_fs, irq, pins.DP, pins.DN, ep_out_buffer, config)
}
