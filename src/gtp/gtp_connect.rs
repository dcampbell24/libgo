//! Connect to a TCP server that knows how to drive GTP engines.

use std::io::prelude::*;
use std::io::BufReader;
use std::net::{SocketAddr, TcpStream};

use game::Game;
use gtp;
use gtp::command::Command;

/// Play Go as a GTP engine waiting for commands.
///
/// Commands are read from stdin and responses are written to stdout.
pub fn play_go(address: SocketAddr) {
    let commands = gtp::register_commands();
    let mut game = Game::new();

    let mut stream = TcpStream::connect(address).expect("failed to bind server to address");

    for line in BufReader::new(stream.try_clone().expect("failed to clone stream")).lines() {
        let line = line.expect("failed to read line");
        println!("<- {}", line);

        if let Some(command) = Command::from_line(&line) {
            let result = gtp::gtp_exec(&mut game, &command, &commands);
            let reply = gtp::command_result::display(command.id, result);
            print!("-> {}", reply);
            stream.write_all(reply.to_owned().as_bytes()).expect("failed to send reply");

            if command.name == "quit" {
                return
            }
        }
    }
}
