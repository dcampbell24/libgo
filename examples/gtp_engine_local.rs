//! A GTP engine that reads from stdin and writes to stdout.

extern crate libgo;

use std::io::{self, BufRead};

use libgo::game::Game;
use libgo::gtp::command::Command;
use libgo::gtp::engine::Engine;

fn main() {
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
