use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

use game::{Game, Handicap};
use game::board::Move;
use game::player::Player;
use game::vertex::Vertex;
use gtp::command::Command;
use gtp::command_result::CommandResult;

/// The library version.
pub const AGENT_VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// The Go Text Protocol (GTP) version.
const GTP_PROTOCOL_VERSION: &'static str = "2";

/// The official name of the agent.
const PROGRAM_NAME: &'static str = env!("CARGO_PKG_NAME");

fn gtp_boardsize(args: &Vec<String>, game: &mut Game) -> CommandResult {
    if args.len() < 1 {
        return Err("boardsize not given".to_owned());
    }

    match u32::from_str_radix(&args[0], 10) {
        Ok(size) => {
            match Game::with_board_size(size as usize) {
                Ok(new_game) => { *game = new_game; Ok(None) },
                Err(_) => Err("unacceptable size".to_owned()),
            }
        },
        Err(_) => Err("boardsize not a u32".to_owned()),
    }
}

fn gtp_genmove(args: &Vec<String>, game: &mut Game) -> CommandResult {
    if args.is_empty() {
        return Err("too few arguments, expected: genmove <color>".to_owned());
    }
    let player = try!(parse_color(&args[0]));
    let move_ = game.genmove_random(player);
    let move_str = match move_.vertex {
        Some(vertex) => vertex.to_string(),
        None => "pass".to_owned(),
    };
    Ok(Some(move_str))
}

fn gtp_place_handicap(args: &Vec<String>,
                      game: &mut Game, handicap: Handicap) -> CommandResult {

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
        for (index, vert) in verts.iter().enumerate() {
            if index == 0 {
                out.push_str(&vert.to_string());
            } else {
                out.push_str(" ");
                out.push_str(&vert.to_string());
            }
        }
        Some(out)
    })
}

fn gtp_play(args: &Vec<String>, game: &mut Game) -> CommandResult {
    if args.len() < 2 {
        return Err("too few arguments, expected: <color> <vertex>".to_owned());
    }

    let color = try!(parse_color(&args[0]));
    let vertex = args[1].to_uppercase();
    if &vertex == "PASS" {
        return game.play(&Move { player: color, vertex: None }).map(|_ok| None);
    }

    let vertex = try!(Vertex::from_str(&vertex));
    if vertex.x >= game.board().size() || vertex.y >= game.board().size() {
        return Err("illegal move".to_owned());
    }

    let mov = Move {
        player: color,
        vertex: Some(vertex),
    };
    return game.play(&mov).map(|_ok| None);
}

fn parse_color(color: &str) -> Result<Player, String> {
    match color.to_lowercase().as_ref() {
        "b" | "black" => Ok(Player::Black),
        "w" | "white" => Ok(Player::White),
        _ => Err(format!("invalid color: {}", color)),
    }
}

/// A structure holding a map of commands to their fns.
pub struct Engine {
    inner: HashMap<String, Box<Fn(&Vec<String>, &mut Game) -> CommandResult>>
}

impl Engine {
    /// Returns whether or not a command is in the map.
    pub fn contains(&self, command: &Command) -> bool {
        if command.args.is_empty() {
            false
        } else {
            self.inner.contains_key(&command.args[0])
        }
    }

    /// Runs the given command with the given game and returns the result.
    pub fn exec(&self, mut game: &mut Game, command: &Command) -> CommandResult {
        match command.name.as_ref() {
            "list_commands" => Ok(Some(self.to_string())),
            "known_command" => Ok(Some(self.contains(command).to_string())),
            _ => self.inner.get(&command.name).map_or(Err("unknown command".to_owned()), |f| {
                f(&command.args, &mut game)
            })
        }
    }

    /// Adds a command to the command map.
    pub fn insert<F>(&mut self, name: &str, f: F)
        where F: 'static + Fn(&Vec<String>, &mut Game) -> CommandResult {

        self.inner.insert(name.to_owned(), Box::new(f));
    }

    /// Returns a new Self containing all of the GTP required commands.
    pub fn new() -> Self {
        let mut commands = Engine { inner: HashMap::new() };

        commands.insert("boardsize", |args, game| {
            gtp_boardsize(args, game)
        });
        commands.insert("clear_board", |_args, game| {
            game.clear_board();
            Ok(None)
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
            args[0].parse::<f64>().ok().map_or(Err("komi is not a float".to_owned()), |komi| {
                game.komi = komi;
                Ok(None)
            })
        });
        commands.insert("list_commands", |_args, _game| {
            unreachable!();
        });
        commands.insert("name", |_args, _game| {
            Ok(Some(PROGRAM_NAME.to_owned()))
        });
        commands.insert("play", |args: &Vec<String>, game: &mut Game| {
            gtp_play(&args, game)
        });
        commands.insert("protocol_version", |_args, _game| {
            Ok(Some(GTP_PROTOCOL_VERSION.to_owned()))
        });
        commands.insert("quit", |_args, _game| {
            Ok(None)
        });
        commands.insert("version", |_args, _game| {
            Ok(Some(AGENT_VERSION.to_owned()))
        });

        commands
    }

    /// Registers all known standard GTP commands.
    pub fn register_all_commands(&mut self) {
        self.register_extra_commands();
        self.register_tournament_commands();
    }

    /// Registers non-standard commands added by David Campbell (DLC).
    pub fn register_dlc_commands(&mut self) {
        self.insert("dlc-debug_game", |_args, game| {
            Ok(Some(format!("{:#?}", game)))
        });
        self.insert("dlc-game_value", |_args, game| {
            Ok(Some(game.value().to_string()))
        });
    }

    /// Register additional GTP commands that are not required.
    pub fn register_extra_commands(&mut self) {
        // Core Play Command
        self.insert("undo", |_args, game| {
            match game.undo() {
                Ok(()) => Ok(None),
                Err(_) => Err("cannot undo".to_owned()),
            }
        });
        // Debug Command
        self.insert("showboard", |_args, game| {
            Ok(Some(format!("\r\n{}", game.board())))
        });

        // Tournament Commands
        // final_score
        // final_status_list
        // time_left
        // time_settings
    }

    /// Registers commands specific to playing on KGS.
    pub fn register_kgs_commands(&mut self) {
        // kgs-chat
        self.insert("kgs-game_over", |_args, game| {
            game.kgs_game_over = true;
            Ok(None)
        });
        self.insert("kgs-genmove_cleanup", |args, game| {
            gtp_genmove(&args, game)
        });
        // kgs-rules
        // kgs-time_settings
    }

    /// Not Supported! Registers commands useful for GTP regression testing.
    pub fn register_regression_commands(&mut self) {
        unimplemented!();
        // loadsgf
        // reg_genmove
    }

    /// Registers the commands required by GTP for tournament play.
    pub fn register_tournament_commands(&mut self) {
        self.insert("fixed_handicap", |args, game| {
            gtp_place_handicap(args, game, Handicap::Fixed)
        });
        self.insert("place_free_handicap", |args, game| {
            gtp_place_handicap(args, game, Handicap::Free)
        });
        self.insert("set_free_handicap", |args, game| {
            let verts: HashSet<_> = args.iter().filter_map(|s| {
                Vertex::from_str(&s.to_uppercase()).ok()
            }).collect();
            if verts.len() != args.len() {
                return Err("syntax error, repeated vertex, or pass given as argument".to_owned());
            }

            game.set_free_handicap(verts).map(|_ok| None)
        });
    }
}

impl fmt::Debug for Engine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut commands: Vec<_> = self.inner.keys().map(|s: &String| s.to_owned()).collect();
        commands.sort();
        write!(f, "\r\n{}", &commands.join("\r\n"))
    }
}
