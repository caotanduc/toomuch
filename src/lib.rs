//! # toomuch
//!
//! A GNU `timeout`-compatible command wrapper with interactive suspend/resume support.
//!
//! This crate provides both:
//! - A reusable library
//! - A `toomuch` CLI binary

pub mod job;
pub mod prompt;
pub mod signal;
pub mod terminal;
pub mod timeout;
pub mod ui;
