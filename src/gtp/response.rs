use std::fmt;

const EOL: &'static str = "\r\n";

/// The GTP result of issuing a command to a driver.
pub type CommandResult = Result<Option<String>, String>;

/// A Go Text Protocol response.
#[derive(Debug)]
pub struct Response {
    /// A sequence id.
    pub id: Option<u32>,
    /// The result of running the command.
    pub result: CommandResult,
}

impl fmt::Display for Response {
    /// Returns a properly formatted GTP response.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let command_id = self.id.map_or(String::new(), |id| id.to_string());
        match self.result {
            Ok(Some(ref reply)) => write!(f, "={} {}{eol}{eol}", command_id, reply, eol = EOL),
            Ok(None) => write!(f, "={} {eol}{eol}", command_id, eol = EOL),
            Err(ref error) => write!(f, "?{} {}{eol}{eol}", command_id, error, eol = EOL),
        }
    }
}
