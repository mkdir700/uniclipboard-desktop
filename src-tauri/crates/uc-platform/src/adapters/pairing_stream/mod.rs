//! Pairing stream utilities.
//!
//! Currently exposes framing helpers for libp2p stream transport.

pub mod framing;
pub mod service;

#[cfg(test)]
mod service_test;
