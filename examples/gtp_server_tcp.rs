extern crate clap;

use std::io::prelude::*;
use std::io::BufReader;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

use clap::{App, Arg};

static VERSION: &str = "0.1.0-dev";

fn main() {
    let matches = App::new("A Go Server")
        .about(
            "\nThis is a TCP server that listens for GTP engines\
             to connect and then plays them against each other.",
        )
        .version(VERSION)
        .arg(
            Arg::with_name("<host:port>")
                .index(1)
                .help("Listen for GTP drivers on host and port"),
        )
        .arg(
            Arg::with_name("N")
                .long("boardsize")
                .takes_value(true)
                .help("Send 'boardsize N' to clients"),
        )
        .get_matches();

    let mut setup_commands = Vec::new();
    if let Some(size) = matches.value_of("N") {
        setup_commands.push(format!("boardsize {}\n", size));
    }

    if let Some(address) = matches.value_of("<host:port>") {
        start(address, setup_commands)
    } else {
        start("127.0.0.1:8000", setup_commands)
    }
}

struct Game {
    black_connection: TcpStream,
    white_connection: TcpStream,
}

fn send_command(
    command: &str,
    writer: &mut TcpStream,
    reader: &mut BufReader<TcpStream>,
) -> String {
    print!("-> {}", command);
    writer.write_all(command.as_bytes()).unwrap();

    let mut reply = String::new();
    reader.read_line(&mut reply).unwrap();
    reader.read_line(&mut reply).unwrap();
    print!("<- {}", &reply);
    reply
}

impl Game {
    fn start(&mut self, setup_commands: Vec<String>) {
        let mut black_reader = BufReader::new(self.black_connection.try_clone().unwrap());
        let mut white_reader = BufReader::new(self.white_connection.try_clone().unwrap());

        for command in setup_commands {
            send_command(&command, &mut self.black_connection, &mut black_reader);
            send_command(&command, &mut self.white_connection, &mut white_reader);
        }

        for i in 1..362 {
            println!("*** turn {:04} ***", 2 * i - 1);
            let black_move =
                send_command("genmove b\n", &mut self.black_connection, &mut black_reader);
            send_command(
                &black_move.replace('=', "play b").replace("\n\n", "\n"),
                &mut self.white_connection,
                &mut white_reader,
            );

            println!("*** turn {:04} ***", 2 * i);
            let white_move =
                send_command("genmove w\n", &mut self.white_connection, &mut white_reader);
            send_command(
                &white_move.replace('=', "play w").replace("\n\n", "\n"),
                &mut self.black_connection,
                &mut black_reader,
            );

            if black_move == "= pass\n\n" && white_move == "= pass\n\n" {
                break;
            }
        }

        self.black_connection.shutdown(Shutdown::Both).unwrap();
        self.white_connection.shutdown(Shutdown::Both).unwrap();
    }
}

fn start(address: &str, setup_commands: Vec<String>) {
    let listener = TcpListener::bind(address).unwrap();
    println!("listening on {} ...", address);

    let mut players = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if players.is_empty() {
                    players.push(stream);
                } else {
                    let mut game = Game {
                        black_connection: players.pop().unwrap(),
                        white_connection: stream,
                    };
                    let setup_commands = setup_commands.clone();
                    thread::spawn(move || {
                        game.start(setup_commands);
                    });
                }
            }
            Err(_e) => { /* connection failed */ }
        }
    }
}
