use std::fmt;

const EOL: &str = "\r\n";

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
            Ok(Some(ref reply)) => write!(f, "={command_id} {reply}{EOL}{EOL}"),
            Ok(None) => write!(f, "={command_id} {EOL}{EOL}"),
            Err(ref error) => write!(f, "?{command_id} {error}{EOL}{EOL}"),
        }
    }
}
