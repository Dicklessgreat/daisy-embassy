//! This example shows how to make a USB MIDI class device and echoing incoming MIDI messages back to the host.
//! The example doesn't include any parsing of the MIDI data, just showing the basic data transport.
#![no_std]
#![no_main]

use defmt::{panic, *};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usb::{Config, Driver, Instance};
use embassy_usb::class::midi::MidiClass;
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;
use {defmt_rtt as _, panic_probe as _};

// If you are trying this and your USB device doesn't connect, the most
// common issues are the RCC config and vbus_detection
//
// See https://embassy.dev/book/#_the_usb_examples_are_not_working_on_my_board_is_there_anything_else_i_need_to_configure
// for more information.
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let p = embassy_stm32::init(daisy_embassy::default_rcc());

    let board = daisy_embassy::new_daisy_board!(p);

    let mut config = Config::default();
    // Do not enable vbus_detection. This is a safe default that works in all boards.
    // However, if your USB device is self-powered (can stay powered on if USB is unplugged), you need
    // to enable vbus_detection to comply with the USB spec. If you enable it, the board
    // has to support it or USB won't work at all. See docs on `vbus_detection` for details.
    config.vbus_detection = false;

    let driver = board.usb_peripherals.build(config);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xdead, 0xc0de);
    config.manufacturer = Some("Daisy-Embassy");
    config.product = Some("USB-MIDI example");
    config.serial_number = Some("12345678");

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder.
    let mut class = MidiClass::new(&mut builder, 1, 1, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Use the Midi class!
    let midi_fut = async {
        loop {
            class.wait_connection().await;
            info!("Connected");
            let _ = midi_echo(&mut class).await;
            info!("Disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, midi_fut).await;
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn midi_echo<'d, T: Instance + 'd>(
    class: &mut MidiClass<'d, Driver<'d, T>>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}
