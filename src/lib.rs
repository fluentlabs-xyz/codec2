extern crate prettytable;

// re-export prettytable macros
#[macro_use]
pub mod macros {
    pub use prettytable::cell;
    pub use prettytable::row;
    pub use prettytable::table;
}

pub mod empty;
pub mod encoder;
pub mod evm;
pub mod primitive;
pub mod utils;
pub mod vec;

// mod encoder;

// pub mod encoder;
// pub mod encoder3;

// mod hash;
// pub mod tuple;

// #[cfg(test)]
// mod tests;
