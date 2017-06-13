/// The GTP result of issuing a command to a driver.
pub type CommandResult = Result<Option<String>, String>;

const EOL: &'static str = "\r\n";

/// Returns a properly formatted GTP response.
pub fn display(id: Option<u32>, result: CommandResult) -> String {
    let command_id = id.map_or(String::new(), |id| id.to_string());
    match result {
       Ok(Some(reply)) => {
           format!("={} {}{eol}{eol}", command_id, reply, eol=EOL)
       },
       Ok(None) => {
           format!("={} {eol}{eol}", command_id, eol=EOL)
       }
       Err(error) => {
           format!("?{} {}{eol}{eol}", command_id, error, eol=EOL)
       }
   }
}
