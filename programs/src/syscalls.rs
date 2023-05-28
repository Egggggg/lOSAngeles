//! Syscall ABI:
//!   On Call:
//!     RAX - Syscall number
//!     RDI, RSI, RDX, R8, R9, R10 - Args, first to last
//!   On Return:
//!     RAX - Status code
//!     RDI, RSI, RDX, R8, R9, R10 - Return values, first to last

pub use serial::*;
pub use graphics::*;

mod serial;
mod graphics;