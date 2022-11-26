//! # libgo
//!
//! A basic Go library that implements the Go Text Protocol. It contains two sub-modules: `game` and
//! `gtp`. `gtp` contans logic for implementing the Go Text Protocol and `game` contains core game
//! logic.

#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

extern crate rand;

pub mod game;
pub mod gtp;
