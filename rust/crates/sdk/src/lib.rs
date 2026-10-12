//! GJC SDK wire transport implementation.
//!
//! This crate provides a Rust implementation of the GJC SDK wire protocol,
//! compatible with the gajae-code SDK broker framing, request/response correlation,
//! and event stream semantics.

#![forbid(unsafe_code)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions
)]

pub mod error;
pub mod frame;
pub mod transport;

pub use error::{SdkError, SdkResult};
pub use frame::{Frame, FrameType};
pub use transport::{SdkTransport, SdkTransportConfig};
