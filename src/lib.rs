#![doc = include_str!("../README.md")]

pub mod ast;
pub mod errors;
pub mod eval;
pub mod parser;
pub mod repl;
pub mod runtime;
pub mod utils;
pub mod warning;

pub use bigdecimal::num_bigint::BigInt;
pub use bigdecimal::BigDecimal;
pub use errors::Error;
pub use warning::Warning;
