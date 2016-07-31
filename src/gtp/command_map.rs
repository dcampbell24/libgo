use std::collections::HashMap;
use std::fmt;

use game::Game;
use gtp::command::Command;
use gtp::command_result::CommandResult;

/// A structure holding a map of commands to their fns.
pub struct CommandMap {
    inner: HashMap<String, Box<Fn(&Vec<String>, &mut Game) -> CommandResult>>
}

impl CommandMap {
    /// Returns a new CommandMap.
    pub fn new() -> Self {
        CommandMap { inner: HashMap::new() }
    }

    /// Returns whether or not a command is in the map.
    pub fn contains(&self, command: &Command) -> bool {
        if command.args.is_empty() {
            false
        } else {
            self.inner.contains_key(&command.args[0])
        }
    }

    /// Adds a command to the command map.
    pub fn insert<F>(&mut self, name: &str, f: F)
        where F: 'static + Fn(&Vec<String>, &mut Game) -> CommandResult {

        self.inner.insert(name.to_owned(), Box::new(f));
    }

    /// Runs the given command with the given game and returns the result.
    pub fn exec(&self, mut game: &mut Game, command: &Command) -> CommandResult {
        match self.inner.get(&command.name) {
            Some(f) => f(&command.args, &mut game),
            None => Err("unknown command".to_owned()),
        }
    }
}

impl fmt::Debug for CommandMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for CommandMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut commands: Vec<_> = self.inner.keys().map(|s: &String| s.to_owned()).collect();
        commands.sort();
        write!(f, "\r\n{}", &commands.join("\r\n"))
    }
}
