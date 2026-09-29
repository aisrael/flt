//! A struct to hold and manage `/command`s

use std::collections::HashMap;

/// A struct to hold and manage `/command`s
#[derive(Default)]
pub struct SlashCommands {
    commands: Vec<SlashCommand>,
    lookup: HashMap<String, usize>,
}

impl SlashCommands {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_command(&mut self, command: SlashCommand) {
        let index = self.commands.len();
        self.lookup.insert(command.long.clone(), index);
        if let Some(short) = command.short.as_ref() {
            self.lookup.insert(short.to_string(), index);
        }
        self.commands.push(command);
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

/// A single slash command
pub struct SlashCommand {
    pub long: String,
    pub short: Option<char>,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slash_commands_lookup() {
        let mut slash_commands = SlashCommands::new();
        let command = SlashCommand {
            long: "foo".to_string(),
            short: Some('f'),
            description: None,
        };
        slash_commands.add_command(command);
        assert_eq!(1, slash_commands.len());
    }
}
