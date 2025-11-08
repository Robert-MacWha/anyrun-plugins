use std::process::Command;

use anyrun_plugin::HandleResult;

use crate::{Matcher, SimpleMatch};

/// Basic copy handler that runs the matched description as a command.
pub struct CommandDisplayHandler {
    command: String,
}

impl CommandDisplayHandler {
    pub fn new(command: &str) -> Self {
        CommandDisplayHandler {
            command: command.to_string(),
        }
    }
}

impl Matcher for CommandDisplayHandler {
    fn get_matches(&self, _text: Vec<&str>) -> Vec<SimpleMatch> {
        let cmd = &self.command;
        let output = Command::new("sh").arg("-c").arg(cmd).output();

        let output = match output {
            Ok(output) => output,
            Err(e) => {
                return vec![SimpleMatch::new(
                    "error",
                    "dialog-error",
                    &format!("Failed to execute command: {}", e),
                )];
            }
        };

        return vec![SimpleMatch::new(
            cmd,
            "",
            &String::from_utf8_lossy(&output.stdout),
        )];
    }

    fn handle(&self, _selection: SimpleMatch) -> HandleResult {
        HandleResult::Close
    }
}
