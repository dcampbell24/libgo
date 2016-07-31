
fn preprocess_line(line: &str) -> Vec<String> {
    let mut out = String::new();
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
        out.push(c);
    }

    out.split_whitespace().map(|s| s.to_owned()).collect()
}

/// A GTP command.
#[derive(Clone,Debug)]
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
    pub fn from_line(line: &str) -> Option<Self> {
        let words = preprocess_line(&line);
        if words.is_empty() {
            return None;
        }

        let mut command_index = 0;
        let id = u32::from_str_radix(&words[0], 10).ok();
        if id.is_some() {
            command_index += 1;
        }
        if command_index >= words.len() {
            return Some(Command {
                id: id,
                name: String::new(),
                args: Vec::new(),
            });
        }

        Some(Command {
            id: id,
            name: words[command_index].clone(),
            args: words[(command_index + 1)..].to_vec(),
        })
    }
}
