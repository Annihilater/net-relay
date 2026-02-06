//! # Net-Relay API
//!
//! REST API for the net-relay dashboard and monitoring.

pub mod handlers;
pub mod router;

pub use router::create_router;
