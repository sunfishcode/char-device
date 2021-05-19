//! Character Device I/O

#![deny(missing_docs)]
#![cfg_attr(can_vector, feature(can_vector))]
#![cfg_attr(write_all_vectored, feature(write_all_vectored))]

#[cfg(feature = "async-std")]
mod async_std;
mod char_device;
#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "async-std")]
pub use crate::async_std::AsyncStdCharDevice;
pub use crate::char_device::CharDevice;
#[cfg(feature = "tokio")]
pub use crate::tokio::TokioCharDevice;
