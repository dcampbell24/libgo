//! This module implements the [Go Text Protocol][1] with [KGS][2] support.
//
//! [1]: http://www.lysator.liu.se/~gunnar/gtp/
//! [2]: http://www.gokgs.com

/// A Go Text Protocol Command.
pub mod command;
/// A GTP engine that accepts commands and returns reponses.
pub mod engine;
/// The result of executing a Go Text Protocol Command.
pub mod response;
