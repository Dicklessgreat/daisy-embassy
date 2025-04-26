# daisy-embassy

`daisy-embassy` is a Rust crate for building **async audio applications** on the [Daisy Seed](https://electro-smith.com/products/daisy-seed) using the [Embassy framework](https://github.com/embassy-rs/embassy). It provides a streamlined interface to initialize and configure Daisy Seed hardware for low-latency, non-blocking audio processing, making it an **ideal starting point** for embedded audio projects in Rust.

This crate is designed for developers familiar with embedded systems and audio processing, but new to Rust's embedded ecosystem. It enables safe and flexible audio application development, leveraging Rust's type system to prevent common peripheral configuration errors at compile time.

## Key Features

- **Async Audio Processing**: Leverage Rust's `async`/`await` and Embassy's lightweight runtime for efficient audio pipelines.
- **Simplified Setup**: Use the `new_daisy_board!` macro to initialize Daisy Seed peripherals with minimal boilerplate.
- **Safe Configuration**: Get sane clock defaults via `daisy_embassy::default_rcc`, and avoid the usual headaches of manual peripheral and DMA setup.
- **Flexible API**: Access peripherals through builder structs for safe defaults, or dive deeper with public accessors for custom configurations.
- **Community-Inspired**: Built on foundations from [stm32h7xx-hal](https://github.com/stm32-rs/stm32h7xx-hal), [daisy_bsp](https://github.com/antoinevg/daisy_bsp), [zlosynth/daisy](https://github.com/zlosynth/daisy), and [libdaisy-rust](https://github.com/mtthw-meyer/libdaisy-rust).

---

## Quick Start: Audio Passthrough Example

To demonstrate the ease of use, here's a simplified version of the `passthrough.rs` example, which sets up an audio passthrough (input to output) using the `new_daisy_board!` macro:

```rust
// safe clock configuration
let config = daisy_embassy::default_rcc();
// initialize the "board"
let p = hal::init(config);
let board: DaisyBoard<'_> = new_daisy_board!(p);

// build the "interface"
let mut interface = board
    .audio_peripherals
    .prepare_interface(Default::default())
    .await;

// start audio callback
interface
    .start(|input, output| {
        // process audio data
        // here we just copy input to output
        output.copy_from_slice(input);
    })
    .await;
```

### How It Works

- **Macro Simplicity**: The `new_daisy_board!` macro moves necessary objects from `embassy::Peripherals` into builders like `daisy_embassy::AudioPeripherals` or `daisy_embassy::FlashBuilder` and so on, streamlining peripheral initialization.
- **Builder Pattern**: Peripherals are accessed via a `XXXBuilder` struct, which provides builder methods (in the case above, `.prepare_interface()`) for safe configuration.
- **Flexibility**: Builders expose `pub` accessors, allowing advanced users to bypass our building and implement custom initialization logic for peripherals.
- **Safety**: The API ensures memory safety and correct peripheral usage, aligning with Rust's guarantees.

See the `examples/` directory for more demos, such as `blinky.rs` or `triangle_wave_tx.rs`.

---

## Supported Daisy Boards

| Board                | Revision | Codec     | Status         |
|----------------------|----------|-----------|----------------|
| Daisy Seed 1.1       | Rev5     | WM8731    | ✅ Supported   |
| Daisy Seed 1.2       | Rev7     | PCM3060   | ✅ Supported   |
| Daisy Seed (AK4556)  | -        | AK4556    | 🚧 Not yet    |
| Daisy Patch SM       | -        | -         | 🚧 Not yet    |

> **Note**: Additional board support is planned. Contributions are welcome; see the [Issues](https://github.com/Dicklessgreat/daisy-embassy/issues) page for details.

---

## Getting Started

### Prerequisites

- **Rust Toolchain**: Install via [rustup](https://rustup.rs/):

    ```bash
    rustup target add thumbv7em-none-eabihf
    ```

- **probe-rs**: For flashing and debugging, [install probe-rs](https://probe.rs/docs/getting-started/installation/).

- **Daisy Seed**: Supported board (Rev5 or Rev7) and USB cable.

> **Tip**: If probe-rs fails, verify your board connection and check [probe-rs docs](https://probe.rs/docs/overview/about-probe-rs/).

### Setup and Run

1. **Clone the Repository**:

   ```bash
   git clone https://github.com/Dicklessgreat/daisy-embassy.git
   cd daisy-embassy
   ```

2. **Identify Your Board**:
   - Rev5 (WM8731): Default, no extra flags.
   - Rev7 (PCM3060): Use `--features=seed_1_2 --no-default-features`.

3. **Run an Example**:

   ```bash
   # Rev5: Blinky example
   cargo run --example blinky --release

   # Rev7: Triangle wave example
   cargo run --example triangle_wave_tx --features=seed_1_2 --no-default-features --release
   ```

4. **Build and Customize**:
   - Explore `examples/` for demos like `passthrough.rs` or `triangle_wave_tx.rs`.
   - Modify examples to create custom audio applications.
   - Debug issues using probe-rs logs.
   - When you find a bug, need help, or have suggestions, open an [Issue](https://github.com/Dicklessgreat/daisy-embassy/issues).

---

## Resources

- [Daisy](https://daisy.audio/hardware/)
- [Embassy Documentation](https://github.com/embassy-rs/embassy)
- [probe-rs Guide](https://probe.rs/docs/overview/about-probe-rs/)
- [Daisy Community Forum](https://forum.electro-smith.com/) for hardware-related questions.

---

## License

This project is licensed under the [MIT License](LICENSE).
