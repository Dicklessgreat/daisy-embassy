This crate is designed for daisy seed with embassy.

I have referred to the following:
- https://github.com/stm32-rs/stm32h7xx-hal
- https://github.com/antoinevg/daisy_bsp
- https://github.com/zlosynth/daisy
- https://github.com/mtthw-meyer/libdaisy-rust

Run examples with `cargo run --example <example_name>`

Let's discuss:
- design interfaces/methods
- audio buffer. zerocopy? DMA or SAI interrupt?
- u32? f32? for audio callback

Not yet implemented:
- audio inout only or output only.
- much much more

contribution:
welcome!