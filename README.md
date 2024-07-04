This crate is designed for daisy seed with embassy.
Now this crate is on REALLY early stage. 
Just for the enthusiast only.
Because the author doesn't know what he is doing, don't try this at your production.

Current status:
- Audio Output...It does indeed send a sound like that at the requested timing, but it is full of noise and not output correctly.I tried to measure it with an oscilloscope by playing a triangular wave. it looked like a kind of triangular wave, but it is far from a "desired" triangular wave.
- Audio Input...Not yet completed. I tried playing music in the "passthrough" example, but it is only faintly audible in the hard noise.

I have referred to the following:
https://github.com/stm32-rs/stm32h7xx-hal
https://github.com/antoinevg/daisy_bsp

Run examples with `cargo run --example <example_name>`

Tell me how to properly set up:
- clocks
- SAI
- SDRAM
- FMC(and what is it used for??)

Let's discuss:
- design interfaces/methods
- audio buffer. zerocopy? DMA or SAI interrupt?
- u32? f32? for audio callback

Not yet implemented:
- audio inout only or output only.
- much much more

contribution:
welcome!