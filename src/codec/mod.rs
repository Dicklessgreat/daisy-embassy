#[cfg(feature = "seed")]
todo!("implement Codec and Pins for ak4556 seed audio codec");

#[cfg(feature = "seed_1_1")]
mod wm8731;
#[cfg(feature = "seed_1_1")]
pub use wm8731::{Codec, Pins};

#[cfg(feature = "seed_1_2")]
mod pcm3060;
#[cfg(feature = "seed_1_2")]
pub use pcm3060::{Codec, Pins};
