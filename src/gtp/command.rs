use std::io::{self, BufRead, Lines};

/// An Iterator that returns GTP commands.
#[derive(Debug)]
pub struct CommandsIter<B> {
    lines: Lines<B>,
}

/// A trait extending `BufRead` to allow reading GTP commands from any type implementing `BufRead`.
pub trait Commands: BufRead {
    /// A method that returns an iterator over GTP commands.
    fn commands(self) -> CommandsIter<Self>
    where
        Self: Sized,
    {
        CommandsIter {
            lines: self.lines(),
        }
    }
}

impl<C: Commands> Iterator for CommandsIter<C> {
    type Item = io::Result<Command>;

    fn next(&mut self) -> Option<Self::Item> {
        for result_line in self.lines.by_ref() {
            match result_line {
                Ok(line) => {
                    if let Some(command) = Command::from_line(&line) {
                        return Some(Ok(command));
                    }
                }
                Err(err) => return Some(Err(err)),
            }
        }
        None
    }
}

impl<T: BufRead> Commands for T {}

// From the GTP 2 Specification Oct 2002:
//
//     3.1 Preprocessing
//
//     When a command string arrives to an engine, it is expected to perform the
//     following four operations before any further parsing takes place:
//
//     1. Remove all occurrences of CR and other control characters except for HT and LF.
//     2. For each line with a hash sign (#), remove all text following and including this character.
//     3. Convert all occurrences of HT to SPACE.
//     4. Discard any empty or white-space only lines.
//
//     When a response arrives to a controller, it is expected only to do steps 1 and 3 above.
//
// We assume input is parsed one line at a time rather than include special newline logic here.
fn preprocess_line(line: &str) -> Option<String> {
    let mut out = String::new();
    let mut keep = false;

    for c in line.chars() {
        if c == '#' {
            break;
        }
        if c == '\t' {
            out.push(' ');
            continue;
        }
        if c.is_control() {
            continue;
        }
        if !keep && !c.is_whitespace() {
            keep = true;
        }
        out.push(c);
    }

    if keep {
        Some(out)
    } else {
        None
    }
}

/// A GTP command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    /// A sequence id.
    pub id: Option<u32>,
    /// The name.
    pub name: String,
    /// Arguments to the command.
    pub args: Vec<String>,
}

impl Command {
    /// Converts a line of input into a Command. Returns None if there was no command.
    #[must_use]
    pub fn from_line(line: &str) -> Option<Self> {
        let mut id = None;
        let mut name = String::new();

        preprocess_line(line).map(|line| {
            let mut words = line.split_whitespace().peekable();
            if let Some(Ok(command_id)) = words.peek().map(|word| word.parse::<u32>()) {
                words.next();
                id = Some(command_id);
            }

            // If there was just an id, we will end up with a malformed command, but the GTP
            // specification does not say how to handle this, so just create the command anyway and
            // let it fail later with "unknown command".

            if let Some(command_name) = words.next() {
                name.push_str(command_name);
            }

            let args: Vec<String> = words.map(ToOwned::to_owned).collect();

            Command { id, name, args }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::u32;

    use super::*;

    #[test]
    fn preprocess_line_() {
        assert_eq!(preprocess_line("1\r2\r quit"), Some("12 quit".to_string()));
        assert_eq!(
            preprocess_line("quit # a comment"),
            Some("quit ".to_string())
        );
        assert_eq!(preprocess_line("\tquit\t"), Some(" quit ".to_string()));
        assert_eq!(preprocess_line("   # a comment"), None);
        assert_eq!(preprocess_line(""), None);
    }

    #[test]
    fn from_line() {
        assert_eq!(Command::from_line(""), None);
        assert_eq!(
            Command::from_line("1"),
            Some(Command {
                id: Some(1),
                name: String::new(),
                args: Vec::new(),
            })
        );
        assert_eq!(
            Command::from_line("1 quit"),
            Some(Command {
                id: Some(1),
                name: "quit".to_string(),
                args: Vec::new(),
            })
        );
        assert_eq!(
            Command::from_line("quit"),
            Some(Command {
                id: None,
                name: "quit".to_string(),
                args: Vec::new(),
            })
        );

        // If a number is given that is not a u32, it will be treated as a command name.
        let large_uint = (u32::MAX as u64 + 1).to_string();
        assert_eq!(
            Command::from_line(&large_uint),
            Some(Command {
                id: None,
                name: large_uint,
                args: Vec::new(),
            })
        );
        let negative_number = "-1".to_string();
        assert_eq!(
            Command::from_line(&negative_number),
            Some(Command {
                id: None,
                name: negative_number,
                args: Vec::new(),
            })
        );

        assert_eq!(
            Command::from_line("play w b19"),
            Some(Command {
                id: None,
                name: "play".to_string(),
                args: vec!["w".to_string(), "b19".to_string()],
            })
        );
    }

    #[test]
    fn commands_() {
        let mut commands = b"one\n2 two\n".commands();
        assert_eq!(
            commands.next().unwrap().unwrap(),
            Command {
                id: None,
                name: "one".to_string(),
                args: Vec::new(),
            }
        );
        assert_eq!(
            commands.next().unwrap().unwrap(),
            Command {
                id: Some(2),
                name: "two".to_string(),
                args: Vec::new(),
            }
        );
    }
}
