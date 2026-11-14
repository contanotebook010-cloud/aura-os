//! Núcleo do aura-dl, sem interface. O binário (CLI hoje, GTK depois) e a
//! ponte do navegador usam estes módulos.

pub mod backend;
pub mod error;
pub mod input;

pub use error::{Error, Result};
