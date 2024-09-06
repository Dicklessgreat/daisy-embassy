use embassy_stm32 as hal;
use hal::peripherals::*;

use defmt::{info, unwrap};
use embassy_time::Timer;

use crate::audio::Fs;

/// A simple HAL for the Cirrus Logic/ Wolfson WM8731 audio codec
pub struct Codec {}

impl Codec {
    //====================wm8731 register set up functions============================
    pub async fn setup_wm8731<'a>(i2c: &mut hal::i2c::I2c<'a, hal::mode::Blocking>, fs: Fs) {
        use wm8731::WM8731;
        info!("setup wm8731 from I2C");

        Timer::after_micros(10).await;

        // reset
        Self::write_wm8731_reg(i2c, WM8731::reset());
        Timer::after_micros(10).await;

        // wakeup
        Self::write_wm8731_reg(
            i2c,
            WM8731::power_down(|w| {
                Self::final_power_settings(w);
                //output off before start()
                w.output().power_off();
            }),
        );
        Timer::after_micros(10).await;

        // disable input mute, set to 0dB gain
        Self::write_wm8731_reg(
            i2c,
            WM8731::left_line_in(|w| {
                w.both().enable();
                w.mute().disable();
                w.volume().nearest_dB(0);
            }),
        );
        Timer::after_micros(10).await;

        // sidetone off; DAC selected; bypass off; line input selected; mic muted; mic boost off
        Self::write_wm8731_reg(
            i2c,
            WM8731::analog_audio_path(|w| {
                w.sidetone().disable();
                w.dac_select().select();
                w.bypass().disable();
                w.input_select().line_input();
                w.mute_mic().enable();
                w.mic_boost().disable();
            }),
        );
        Timer::after_micros(10).await;

        // disable DAC mute, deemphasis for 48k
        Self::write_wm8731_reg(
            i2c,
            WM8731::digital_audio_path(|w| {
                w.dac_mut().disable();
                w.deemphasis().frequency_48();
            }),
        );
        Timer::after_micros(10).await;

        // nothing inverted, slave, 24-bits, MSB format
        Self::write_wm8731_reg(
            i2c,
            WM8731::digital_audio_interface_format(|w| {
                w.bit_clock_invert().no_invert();
                w.master_slave().slave();
                w.left_right_dac_clock_swap().right_channel_dac_data_right();
                w.left_right_phase().data_when_daclrc_low();
                w.bit_length().bits_24();
                w.format().left_justified();
            }),
        );
        Timer::after_micros(10).await;

        // no clock division, normal mode
        Self::write_wm8731_reg(
            i2c,
            WM8731::sampling(|w| {
                w.core_clock_divider_select().normal();
                w.base_oversampling_rate().normal_256();
                match fs {
                    Fs::Fs8000 => {
                        w.sample_rate().adc_8();
                    }
                    Fs::Fs32000 => {
                        w.sample_rate().adc_32();
                    }
                    Fs::Fs44100 => {
                        w.sample_rate().adc_441();
                    }
                    Fs::Fs48000 => {
                        w.sample_rate().adc_48();
                    }
                    Fs::Fs88200 => {
                        w.sample_rate().adc_882();
                    }
                    Fs::Fs96000 => {
                        w.sample_rate().adc_96();
                    }
                }
                w.usb_normal().normal();
            }),
        );
        Timer::after_micros(10).await;

        // set active
        Self::write_wm8731_reg(i2c, WM8731::active().active());
        Timer::after_micros(10).await;

        //Note: WM8731's output not yet enabled.
    }

    pub fn write_wm8731_reg(i2c: &mut hal::i2c::I2c<'_, hal::mode::Blocking>, r: wm8731::Register) {
        const AD: u8 = 0x1a; // or 0x1b if CSB is high

        // WM8731 has 16 bits registers.
        // The first 7 bits are for the addresses, and the rest 9 bits are for the "value"s.
        // Let's pack wm8731::Register into 16 bits.
        let byte1: u8 = ((r.address << 1) & 0b1111_1110) | (((r.value >> 8) & 0b0000_0001) as u8);
        let byte2: u8 = (r.value & 0b1111_1111) as u8;
        unwrap!(i2c.blocking_write(AD, &[byte1, byte2]));
    }

    pub fn final_power_settings(w: &mut wm8731::power_down::PowerDown) {
        w.power_off().power_on();
        w.clock_output().power_off();
        w.oscillator().power_off();
        w.output().power_on();
        w.dac().power_on();
        w.adc().power_on();
        w.mic().power_off();
        w.line_input().power_on();
    }
}

#[allow(non_snake_case)]
pub struct Pins {
    pub SCL: PH4,    // I2C SCL
    pub SDA: PB11,   // I2C SDA
    pub MCLK_A: PE2, // SAI1 MCLK_A
    pub SCK_A: PE5,  // SAI1 SCK_A
    pub FS_A: PE4,   // SAI1 FS_A
    pub SD_A: PE6,   // SAI1 SD_A
    pub SD_B: PE3,   // SAI1 SD_B
}

// ToDo
