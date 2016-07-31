/// The GTP result of issuing a command to a driver.
pub type CommandResult = Result<String, String>;

/// Returns a properly formatted GTP response.
pub fn display(id: Option<u32>, result: CommandResult) -> String {
    let command_id = id.map_or(String::new(), |id| id.to_string());
    match result {
       Ok(reply) => {
           format!("={} {}{eol}{eol}", command_id, reply, eol="\r\n")
       },
       Err(error) => {
           format!("?{} {}{eol}{eol}", command_id, error, eol="\r\n")
       }
   }
}
