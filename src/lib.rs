#![doc = include_str!("../README.md")]

pub mod core;
pub mod helper;
pub mod render;

pub use core::Printer;
pub use render::{Io, Render};
