#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_imports)]
extern crate alloc;

pub mod bytes;
mod empty;
mod encoder;
mod error;
mod evm;
mod hash;
mod primitive;
mod tuple;
mod vec;

pub use encoder::*;
pub use error::*;

// #[cfg(test)]
// mod tests;

#[cfg(feature = "derive")]
extern crate codec_derive;

#[cfg(feature = "derive")]
pub use codec_derive::Codec;
