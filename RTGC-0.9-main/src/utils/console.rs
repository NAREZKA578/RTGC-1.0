//! Console System - Simple in-game console for debug commands

use std::collections::HashMap;
use tracing::info;

/// Console command callback type
pub type CommandCallback = Box<dyn Fn(&[String]) -> Result<String, String> + Send + Sync>;

/// Console command definition
pub struct ConsoleCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub callback: CommandCallback,
}

/// In-game console system
pub struct Console {
    /// Registered commands
    commands: HashMap<String, ConsoleCommand>,
    /// Command history
    history: Vec<String>,
    /// History index for navigation
    history_index: Option<usize>,
    /// Is console visible
    visible: bool,
    /// Current input buffer
    input_buffer: String,
    /// Output lines
    output: Vec<String>,
    /// Max output lines
    max_output_lines: usize,
}

impl Console {
    pub fn new() -> Self {
        let mut console = Self {
            commands: HashMap::new(),
            history: Vec::new(),
            history_index: None,
            visible: false,
            input_buffer: String::new(),
            output: Vec::new(),
            max_output_lines: 100,
        };

        // Register built-in commands
        console.register_built_in_commands();

        console
    }

    fn register_built_in_commands(&mut self) {
        // Help command
        self.register_command(
            "help",
            "Show list of commands or help for a specific command",
            "help [command]",
            Box::new(|args| {
                if args.is_empty() {
                    Ok("Available commands: spawn, set, list, clear, help".to_string())
                } else {
                    Ok(format!("Help for: {}", args[0]))
                }
            }),
        );

        // Spawn vehicle command
        self.register_command(
            "spawn",
            "Spawn a vehicle or object",
            "spawn <type> [x] [y] [z]",
            Box::new(|args| {
                if args.is_empty() {
                    return Err("Usage: spawn <type> [x] [y] [z]".to_string());
                }
                let vehicle_type = &args[0];
                let pos = if args.len() >= 4 {
                    format!(" at ({}, {}, {})", args[1], args[2], args[3])
                } else {
                    " at player position".to_string()
                };
                Ok(format!("Spawning {}{}", vehicle_type, pos))
            }),
        );

        // Set skill command
        self.register_command(
            "set",
            "Set a value (skill, stat, etc.)",
            "set <skill|stat> <value>",
            Box::new(|args| {
                if args.len() < 2 {
                    return Err("Usage: set <skill|stat> <value>".to_string());
                }
                let skill = &args[0];
                let value = &args[1];
                Ok(format!("Set {} to {}", skill, value))
            }),
        );

        // List command
        self.register_command(
            "list",
            "List entities, items, or other objects",
            "list <entities|items|vehicles>",
            Box::new(|args| {
                if args.is_empty() {
                    return Err("Usage: list <entities|items|vehicles>".to_string());
                }
                Ok(format!("Listing {}", args[0]))
            }),
        );

        // Clear command
        self.register_command(
            "clear",
            "Clear console output",
            "clear",
            Box::new(|_| {
                Ok("Console cleared".to_string())
            }),
        );

        // Teleport command
        self.register_command(
            "tp",
            "Teleport to coordinates or entity",
            "tp <x> <y> <z> | <entity>",
            Box::new(|args| {
                if args.is_empty() {
                    return Err("Usage: tp <x> <y> <z> | <entity>".to_string());
                }
                Ok(format!("Teleporting to {:?}", args))
            }),
        );

        // Time command
        self.register_command(
            "time",
            "Set or get game time",
            "time [get|set <hour>]",
            Box::new(|args| {
                if args.is_empty() || args[0] == "get" {
                    Ok("Current time: 12:00".to_string())
                } else if args[0] == "set" && args.len() > 1 {
                    Ok(format!("Time set to {}:00", args[1]))
                } else {
                    Err("Usage: time [get|set <hour>]".to_string())
                }
            }),
        );

        // Weather command
        self.register_command(
            "weather",
            "Set weather state",
            "weather <clear|rain|snow|storm>",
            Box::new(|args| {
                if args.is_empty() {
                    return Err("Usage: weather <clear|rain|snow|storm>".to_string());
                }
                Ok(format!("Weather set to {}", args[0]))
            }),
        );

        // Give item command
        self.register_command(
            "give",
            "Give item to player",
            "give <item> [amount]",
            Box::new(|args| {
                if args.is_empty() {
                    return Err("Usage: give <item> [amount]".to_string());
                }
                let default_amount = "1".to_string();
                let amount = args.get(1).unwrap_or(&default_amount);
                Ok(format!("Gave {} x{}", args[0], amount))
            }),
        );

        // God mode command
        self.register_command(
            "god",
            "Toggle god mode",
            "god",
            Box::new(|_| {
                Ok("God mode toggled".to_string())
            }),
        );

        // FPS command
        self.register_command(
            "fps",
            "Show/set FPS limit",
            "fps [limit]",
            Box::new(|args| {
                if args.is_empty() {
                    Ok("Current FPS: 60".to_string())
                } else {
                    Ok(format!("FPS limit set to {}", args[0]))
                }
            }),
        );
    }

    /// Register a new command
    pub fn register_command(
        &mut self,
        name: &str,
        description: &str,
        usage: &str,
        callback: CommandCallback,
    ) {
        self.commands.insert(
            name.to_lowercase(),
            ConsoleCommand {
                name: name.to_string(),
                description: description.to_string(),
                usage: usage.to_string(),
                callback,
            },
        );
        info!("Registered console command: {}", name);
    }

    /// Toggle console visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.input_buffer.clear();
            self.history_index = None;
        }
    }

    /// Set console visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        if visible {
            self.input_buffer.clear();
            self.history_index = None;
        }
    }

    /// Check if console is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Process a character input
    pub fn input_char(&mut self, c: char) {
        if self.visible {
            self.input_buffer.push(c);
        }
    }

    /// Handle special key input
    pub fn input_key(&mut self, key: ConsoleKey) {
        if !self.visible {
            return;
        }

        match key {
            ConsoleKey::Backspace => {
                self.input_buffer.pop();
            }
            ConsoleKey::Enter => {
                self.submit_command();
            }
            ConsoleKey::Escape => {
                self.set_visible(false);
            }
            ConsoleKey::Up => {
                if let Some(idx) = self.history_index {
                    if idx > 0 {
                        self.history_index = Some(idx - 1);
                        if let Some(cmd) = self.history.get(idx - 1) {
                            self.input_buffer = cmd.clone();
                        }
                    }
                } else if !self.history.is_empty() {
                    self.history_index = Some(self.history.len() - 1);
                    if let Some(cmd) = self.history.last() {
                        self.input_buffer = cmd.clone();
                    }
                }
            }
            ConsoleKey::Down => {
                if let Some(idx) = self.history_index {
                    if idx < self.history.len() - 1 {
                        self.history_index = Some(idx + 1);
                        if let Some(cmd) = self.history.get(idx + 1) {
                            self.input_buffer = cmd.clone();
                        }
                    } else {
                        self.history_index = None;
                        self.input_buffer.clear();
                    }
                }
            }
            ConsoleKey::Left | ConsoleKey::Right => {
                // Left/Right arrow keys - could be used for cursor navigation
                // Currently not implemented
            }
        }
    }

    /// Submit the current command
    fn submit_command(&mut self) {
        let input = self.input_buffer.trim().to_string();
        
        if input.is_empty() {
            return;
        }

        // Add to history
        self.history.push(input.clone());
        if self.history.len() > 50 {
            self.history.remove(0);
        }
        self.history_index = None;

        // Parse command and arguments
        let parts: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
        
        if parts.is_empty() {
            return;
        }

        let command_name = parts[0].to_lowercase();
        let args = parts[1..].to_vec();

        // Execute command
        match self.commands.get(&command_name) {
            Some(command) => {
                match (command.callback)(&args) {
                    Ok(result) => {
                        self.add_output(&format!("> {}", input));
                        self.add_output(&result);
                    }
                    Err(error) => {
                        self.add_output(&format!("> {}", input));
                        self.add_output(&format!("Error: {}", error));
                    }
                }
            }
            None => {
                self.add_output(&format!("> {}", input));
                self.add_output(&format!("Unknown command: {}. Type 'help' for available commands.", command_name));
            }
        }

        self.input_buffer.clear();
    }

    /// Add output line
    fn add_output(&mut self, line: &str) {
        self.output.push(line.to_string());
        while self.output.len() > self.max_output_lines {
            self.output.remove(0);
        }
    }

    /// Get output lines for rendering
    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    /// Get current input buffer
    pub fn get_input(&self) -> &str {
        &self.input_buffer
    }

    /// Get all registered commands
    pub fn get_commands(&self) -> Vec<&String> {
        self.commands.keys().collect()
    }

    /// Get command help
    pub fn get_command_help(&self, name: &str) -> Option<(&str, &str, &str)> {
        self.commands.get(&name.to_lowercase()).map(|cmd| {
            (cmd.name.as_str(), cmd.description.as_str(), cmd.usage.as_str())
        })
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

/// Console special keys
#[derive(Debug, Clone, Copy)]
pub enum ConsoleKey {
    Backspace,
    Enter,
    Escape,
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_basic() {
        let mut console = Console::new();
        
        assert!(!console.is_visible());
        
        console.toggle();
        assert!(console.is_visible());
        
        console.input_char('h');
        console.input_char('e');
        console.input_char('l');
        console.input_char('p');
        
        assert_eq!(console.get_input(), "help");
    }

    #[test]
    fn test_console_command_execution() {
        let mut console = Console::new();
        console.toggle();
        
        console.input_char('c');
        console.input_char('l');
        console.input_char('e');
        console.input_char('a');
        console.input_char('r');
        console.input_key(ConsoleKey::Enter);
        
        assert!(!console.get_output().is_empty());
    }
}
