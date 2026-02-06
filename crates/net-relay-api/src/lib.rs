//! # Net-Relay API
//!
//! REST API for the net-relay dashboard and monitoring.

pub mod auth;
pub mod handlers;
pub mod router;

pub use auth::{session_auth_middleware, SessionStore};
pub use router::create_router;
