//! This module implements the [Go Text Protocol][1] with [KGS][2] support.
//
//! [1]: http://www.lysator.liu.se/~gunnar/gtp/
//! [2]: http://www.gokgs.com

/// A Go Text Protocol Command.
pub mod command;
/// A map from Go Text Protocol Commands to Rust functions.
pub mod engine;
/// A module for connecting the engine to a server over a TCP connection.
pub mod gtp_connect;
/// The result of executing a Go Text Protocol Command.
pub mod response;

use std::io::{self, BufRead};

use game::Game;
use self::command::Command;
use self::engine::Engine;

/// Play Go as a GTP engine waiting for commands.
///
/// Commands are read from stdin and responses are written to stdout.
pub fn play_go() {
    let mut gtp = Engine::new();
    gtp.register_all_commands();

    let mut game = Game::new();
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        if let Some(command) = Command::from_line(&line.expect("failed to read line")) {
            let response = gtp.exec(&mut game, &command);
            print!("{}", response);

            if command.name == "quit" {
                return
            }
        }
    }
}
