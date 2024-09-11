# daisy_embassy

This crate is designed for [daisy seed](https://electro-smith.com/products/daisy-seed) with [embassy](https://github.com/embassy-rs/embassy).

I have referred to the following:

- <https://github.com/stm32-rs/stm32h7xx-hal>
- <https://github.com/antoinevg/daisy_bsp>
- <https://github.com/zlosynth/daisy>
- <https://github.com/mtthw-meyer/libdaisy-rust>

## Supported Daisy

- Daisy Seed 1.1(rev5, with WM8731)
- Daisy Seed 1.2(rev7, with PCM3060)

not yet support Daisy Seed(with AK4556), Daisy Patch SM.

## Run Examples

The first thing we'd like you to do is to run one of the examples on your Daisy board.

### Prerequisites

You can choose your preferred toolset to run the examples, but we recommend you to install [probe-rs](https://github.com/probe-rs/probe-rs)

### Which Daisy Board You Have?

- If you have `rev5`, it's defaulted.So you don't have to care about options.
- If you have `rev7`, add `--features=seed_1_2 --no-default-features`  option each time you run an example.

### Choose a Example You Want to Run 

You can run examples with `cargo run --example <example_name>`.
