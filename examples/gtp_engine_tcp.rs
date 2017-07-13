//! A GTP engine that connects to a TCP server to drive it.

extern crate libgo;

use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpStream;

use libgo::game::Game;
use libgo::gtp::engine::Engine;
use libgo::gtp::command::Command;

pub fn main() {
    let mut gtp = Engine::new();
    gtp.register_all_commands();

    let mut game = Game::new();

    let address = env::args()
        .nth(1)
        .expect("error: expected server address argument host:port");
    let mut stream = TcpStream::connect(address).expect("failed to bind server to address");

    for line in BufReader::new(stream.try_clone().expect("failed to clone stream")).lines() {
        let line = line.expect("failed to read line");
        println!("<- {}", line);

        if let Some(command) = Command::from_line(&line) {
            let response = gtp.exec(&mut game, &command).to_string();
            print!("-> {}", response);
            stream
                .write_all(response.as_bytes())
                .expect("failed to send reply");

            if command.name == "quit" {
                return;
            }
        }
    }
}
