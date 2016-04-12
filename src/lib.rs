#![feature(core_intrinsics)]
#![feature(question_mark)]

mod utils;
mod tests;
mod error;
mod token;
mod tree;
mod compile_utils;
mod ir;
pub mod compile;

pub use self::compile::compile;
