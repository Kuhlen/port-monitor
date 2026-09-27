//! Pure serial-monitor rules: no IO, no UI.

pub mod error;
pub mod filter;
pub mod line_buffer;
pub mod link;
pub mod send;
pub mod serial;

pub use error::AppError;
