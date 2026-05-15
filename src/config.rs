use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub editor: Option<String>,
    pub commands: HashMap<String, Command>,
}

#[derive(Debug, Deserialize)]
pub struct Command {
    pub path: String,
    pub description: Option<String>,
    pub modules: Option<Vec<Module>>,
}

#[derive(Debug, Deserialize)]
pub struct Module {
    pub name: String,
    pub path: String,
    pub desc: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        let config_path = config_file_path();
        if !config_path.exists() {
            eprintln!("Config file not found: {}", config_path.display());
            eprintln!("Create ~/.config/dot/config.toml");
            std::process::exit(1);
        }
        let content = fs::read_to_string(&config_path)
            .unwrap_or_else(|e| {
                eprintln!("Failed to read {}: {}", config_path.display(), e);
                std::process::exit(1);
            });
        toml::from_str(&content).unwrap_or_else(|e| {
            eprintln!("Failed to parse {}: {}", config_path.display(), e);
            std::process::exit(1);
        })
    }

    pub fn editor(&self) -> &str {
        self.editor.as_deref().unwrap_or("code")
    }

    pub fn resolve_path(s: &str) -> PathBuf {
        let expanded = shellexpand::full(s).unwrap_or_else(|_| s.into());
        PathBuf::from(expanded.as_ref())
    }

    pub fn find_command(&self, name: &str) -> Option<&Command> {
        self.commands.get(name)
    }

    pub fn list_commands(&self) -> Vec<(&String, &Command)> {
        let mut cmds: Vec<_> = self.commands.iter().collect();
        cmds.sort_by_key(|(k, _)| *k);
        cmds
    }
}

fn config_file_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
    PathBuf::from(home).join(".config").join("dot").join("config.toml")
}
