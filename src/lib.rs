#[doc = include_str!("../README.md")]
pub mod ast;
pub mod errors;
pub mod eval;
pub mod parser;
pub mod repl;
pub mod runtime;
pub mod utils;

pub use errors::Error;
