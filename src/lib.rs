extern crate prettytable;

// re-export prettytable macros
#[macro_use]
pub mod macros {
    pub use prettytable::cell;
    pub use prettytable::row;
    pub use prettytable::table;
}

pub mod align;
pub mod encoder;
pub mod primitive;
pub mod utils;

// mod empty;
// mod encoder;

// pub mod encoder;
// pub mod encoder3;

// mod evm;
// mod hash;
// pub mod tuple;
// mod vec;

// #[cfg(test)]
// mod tests;
