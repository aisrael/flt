#[doc = include_str!("../README.md")]
pub mod ast;
pub mod errors;
pub mod parser;

pub use errors::Error;
