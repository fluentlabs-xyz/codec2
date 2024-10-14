#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_imports)]
extern crate alloc;

pub mod bytes;
pub mod empty;
pub mod encoder;
pub mod error;
pub mod evm;
pub mod hash;
pub mod primitive;
pub mod tuple;
pub mod vec;

#[cfg(test)]
mod tests;

#[cfg(feature = "derive")]
extern crate codec_derive;

#[cfg(feature = "derive")]
pub use codec_derive::Codec;
