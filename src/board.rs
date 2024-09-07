use crate::flash::FlashBuilder;
use crate::led::UserLed;
use crate::pins::*;
use crate::usb::UsbPeripherals;
use crate::{audio::AudioPeripherals, sdram::SdRamBuilder};
pub struct DaisyBoard<'a> {
    pub pins: DaisyPins,
    pub user_led: UserLed<'a>,
    pub audio_peripherals: AudioPeripherals,
    pub flash: FlashBuilder,
    pub sdram: SdRamBuilder,
    pub usb_peripherals: UsbPeripherals,
    // on board "BOOT" button.
    pub boot: Boot,
}
