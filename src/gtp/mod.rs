//! This module implements the [Go Text Protocol](http://www.lysator.liu.se/~gunnar/gtp/) with [KGS](http://www.gokgs.com) support.

/// A Go Text Protocol Command.
pub mod command;
/// A GTP engine that accepts commands and returns reponses.
pub mod engine;
/// The result of executing a Go Text Protocol Command.
pub mod response;
