//! Character Device I/O

#![deny(missing_docs)]
#![cfg_attr(can_vector, feature(can_vector))]
#![cfg_attr(write_all_vectored, feature(write_all_vectored))]

mod char_device;

pub use crate::char_device::CharDevice;
