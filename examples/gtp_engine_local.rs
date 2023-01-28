//! A GTP engine that reads from stdin and writes to stdout.

extern crate libgo;

use std::io;

use libgo::game::Game;
use libgo::gtp::command::Commands;
use libgo::gtp::engine::Engine;

fn main() {
    let mut gtp = Engine::new();
    gtp.register_all_commands();

    let mut game = Game::new();
    let stdin = io::stdin();

    for command in stdin.lock().commands() {
        let command = command.expect("failed to read command");
        let response = gtp.exec(&mut game, &command);
        print!("{response}");

        if command.name == "quit" {
            return;
        }
    }
}
