use embassy_stm32 as hal;
use hal::{
    peripherals::USB_OTG_FS,
    usb::{Config, Driver},
};
use static_cell::StaticCell;

use crate::{board::Irqs, pins::USB2Pins};

pub type DaisyUsb = Driver<'static, USB_OTG_FS>;

pub struct UsbPeripherals {
    pub usb_otg_fs: USB_OTG_FS,
    pub pins: USB2Pins,
}

impl UsbPeripherals {
    pub fn build(self) -> DaisyUsb {
        let mut config = Config::default();
        // Do not enable vbus_detection. This is a safe default that works in all boards.
        // However, if your USB device is self-powered (can stay powered on if USB is unplugged), you need
        // to enable vbus_detection to comply with the USB spec. If you enable it, the board
        // has to support it or USB won't work at all. See docs on `vbus_detection` for details.
        config.vbus_detection = false;
        static EP_OUT_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
        let ep_out_buffer = EP_OUT_BUFFER.init([0; 256]);
        Driver::new_fs(
            self.usb_otg_fs,
            Irqs,
            self.pins.DP,
            self.pins.DN,
            ep_out_buffer,
            config,
        )
    }
}
