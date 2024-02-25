#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::WaveletDemo;
pub mod audio;
pub mod worklet;

mod depmod;
pub use depmod::*;