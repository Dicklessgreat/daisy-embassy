use embassy_stm32 as hal;
use hal::{
    peripherals::USB_OTG_FS,
    usb::{Config, Driver},
};
use static_cell::StaticCell;

// use crate::{board::Irqs, pins::USB2Pins};
use crate::pins::USB2Pins;

pub type DaisyUsb = Driver<'static, USB_OTG_FS>;

pub struct UsbPeripherals {
    pub usb_otg_fs: USB_OTG_FS,
    pub pins: USB2Pins,
}

impl UsbPeripherals {
    pub fn build(self, config: Config) -> DaisyUsb {
        static EP_OUT_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
        let ep_out_buffer = EP_OUT_BUFFER.init([0; 256]);
        todo!()
        // Driver::new_fs(
        //     self.usb_otg_fs,
        //     Irqs,
        //     self.pins.DP,
        //     self.pins.DN,
        //     ep_out_buffer,
        //     config,
        // )
    }
}
