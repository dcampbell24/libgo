//! This module implements the [Go Text Protocol][1] with [KGS][2] support.
//
//! [1]: http://www.lysator.liu.se/~gunnar/gtp/
//! [2]: http://www.gokgs.com

/// A Go Text Protocol Command.
pub mod command;
/// A map from Go Text Protocol Commands to Rust functions.
pub mod command_map;
/// The result of executing a Go Text Protocol Command.
pub mod command_result;
pub mod gtp_connect;

/// The library version.
pub const AGENT_VERSION: &'static str = env!("CARGO_PKG_VERSION");
/// The Go Text Protocol (GTP) version.
const GTP_PROTOCOL_VERSION: &'static str = "2";
/// The official name of the agent.
const PROGRAM_NAME: &'static str = env!("CARGO_PKG_NAME");

use std::collections::HashSet;
use std::fmt::Write;
use std::io::{self, BufRead};
use std::str::FromStr;

use game::{Game, Handicap};
use game::board::Move;
use game::player::Player;
use game::vertex::Vertex;
use self::command::Command;
use self::command_map::CommandMap;
use self::command_result::CommandResult;

/// Play Go as a GTP engine waiting for commands.
///
/// Commands are read from stdin and responses are written to stdout.
pub fn play_go() {
    let command_map = register_commands();
    let mut game = Game::new();
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        if let Some(command) = Command::from_line(&line.expect("failed to read line")) {
            let result = gtp_exec(&mut game, &command, &command_map);
            print!("{}", self::command_result::display(command.id, result));

            if command.name == "quit" {
                return
            }
        }
    }
}

/// Update the Game with the gtp_command (and arguments) and return the Result.
pub fn gtp_exec(mut game: &mut Game, command: &Command, commands: &CommandMap) -> CommandResult {
    match command.name.as_ref() {
        "list_commands" => Ok(commands.to_string()),
        "known_command" => Ok(commands.contains(command).to_string()),
        "quit" => Ok(String::new()),
        "dlc-debug_game" => Ok(format!("{:#?}", game)),
        _ => commands.exec(game, command)
    }
}

fn gtp_boardsize(args: &Vec<String>, game: &mut Game) -> Result<(), String> {
    if args.len() < 1 {
        return Err("boardsize not given".to_owned());
    }

    match u32::from_str_radix(&args[0], 10) {
        Ok(size) => {
            match Game::with_board_size(size as usize) {
                Ok(new_game) => { *game = new_game; Ok(()) },
                Err(_) => Err("unacceptable size".to_owned()),
            }
        },
        Err(_) => Err("boardsize not a u32".to_owned()),
    }
}

fn parse_color(color: &str) -> Result<Player, String> {
    match color.to_lowercase().as_ref() {
        "b" | "black" => Ok(Player::Black),
        "w" | "white" => Ok(Player::White),
        _ => Err(format!("invalid color: {}", color)),
    }
}

fn gtp_genmove(args: &Vec<String>, game: &mut Game) -> Result<String, String> {
    if args.is_empty() {
        return Err("too few arguments, expected: genmove <color>".to_owned());
    }
    let player = try!(parse_color(&args[0]));
    let move_ = game.genmove_random(player);
    let move_str = match move_.vertex {
        Some(vertex) => vertex.to_string(),
        None => "pass".to_owned(),
    };
    Ok(move_str)
}

fn gtp_play(args: &Vec<String>, game: &mut Game) -> Result<(), String> {
    if args.len() < 2 {
        return Err("too few arguments, expected: <color> <vertex>".to_owned());
    }

    let color = try!(parse_color(&args[0]));
    let vertex = args[1].to_uppercase();
    if &vertex == "PASS" {
        return game.play(&Move { player: color, vertex: None });
    }

    let vertex = try!(Vertex::from_str(&vertex));
    if vertex.x >= game.board().size() || vertex.y >= game.board().size() {
        return Err("illegal move".to_owned());
    }

    let mov = Move {
        player: color,
        vertex: Some(vertex),
    };
    return game.play(&mov);
}

fn gtp_place_handicap(args: &Vec<String>,
                      game: &mut Game, handicap: Handicap) -> Result<String, String> {

    if args.is_empty() {
        return Err("syntax error".to_owned());
    }
    let stones = match u32::from_str_radix(&args[0], 10) {
        Ok(stones) => stones as usize,
        Err(_) => {
            return Err("number is not a u32".to_owned());
        }
    };
    game.place_handicap(stones, handicap).map(|verts| {
        let mut out = String::new();
        for vert in verts.into_iter() {
            out.push_str(&vert.to_string());
        }
        out
    })
}

/// Register the GTP commands.
pub fn register_commands() -> CommandMap {
    let mut commands: CommandMap = CommandMap::new();

    // GTP version 2 standard commands.
    commands.insert("boardsize", |args, game| {
        gtp_boardsize(args, game).map(|_| String::new())
    });
    commands.insert("clear_board", |_args, game| {
        game.clear_board();
        Ok(String::new())
    });
    commands.insert("dlc-game_value", |_args, game| {
        Ok(game.value().to_string())
    });
    // commands.insert("final_score".to_owned(), Box::new(|args: &Vec<String>, goban: &mut Game| unimplemented!()));
    commands.insert("final_status_list", |_args, _game| {
        unimplemented!();
    });
    commands.insert("fixed_handicap", |args, game| {
        gtp_place_handicap(args, game, Handicap::Fixed)
    });
    commands.insert("genmove", |args, game| {
        gtp_genmove(&args, game)
    });
    commands.insert("known_command", |_args, _game| {
        unreachable!();
    });
    commands.insert("komi", |args, game| {
        if args.is_empty() {
            return Err("expected komi value".to_owned());
        }

        match args[0].parse::<f64>() {
            Ok(komi) => {
                game.komi = komi;
                Ok(String::new())
            },
            Err(_) => {
                Err("komi is not a float".to_owned())
            }
        }
    });
    commands.insert("kgs-game_over", |_args, game| {
        game.kgs_game_over = true;
        Ok(String::new())
    });
    commands.insert("kgs-genmove_cleanup", |args, game| {
        gtp_genmove(&args, game)
    });
    // commands.insert("loadsgf".to_owned(), Box::new(|args: &Vec<String>, goban: &mut Game| unimplemented!()));
    commands.insert("list_commands", |_args, _game| {
        unreachable!();
    });
    commands.insert("name", |_args, _game| {
        Ok(PROGRAM_NAME.to_owned())
    });
    commands.insert("place_free_handicap", |args, game| {
        gtp_place_handicap(args, game, Handicap::Free)
    });
    commands.insert("play", |args: &Vec<String>, game: &mut Game| {
        gtp_play(&args, game).map(|_| String::new())
    });
    commands.insert("protocol_version", |_args, _game| {
        Ok(GTP_PROTOCOL_VERSION.to_owned())
    });
    commands.insert("quit", |_args, _game| {
        unreachable!()
    });
    commands.insert("set_free_handicap", |args, game| {
        let verts: HashSet<_> = args.iter().filter_map(|s| {
            Vertex::from_str(&s.to_uppercase()).ok()
        }).collect();
        if verts.len() != args.len() {
            return Err("syntax error, repeated vertex, or pass given as argument".to_owned());
        }

        game.set_free_handicap(verts).map(|_| String::new())
    });
    commands.insert("showboard", |_args, game| {
        Ok(format!("\r\n{}", game.board()))
    });
    // commands.insert("time_left".to_owned(), Box::new(|args: &Vec<String>, goban: &mut Game| unimplemented!()));
    // commands.insert("time_settings".to_owned(), Box::new(|args: &Vec<String>, goban: &mut Game| unimplemented!()));
    commands.insert("undo", |_args, game| {
        match game.undo() {
            Ok(()) => Ok(String::new()),
            Err(_) => Err("cannot undo".to_owned()),
        }
    });
    commands.insert("version", |_args, _game| {
        Ok(AGENT_VERSION.to_owned())
    });

    commands
}
