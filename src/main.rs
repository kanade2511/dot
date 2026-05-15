mod config;

use clap::Parser;
use config::Config;
use std::process::Command;

#[derive(Parser)]
#[command(name = "dot", version, about = "Quickly open config files")]
struct Cli {
    #[arg(short = 'l', long = "list", help = "List all available commands")]
    list: bool,

    #[arg(help = "Config command (e.g., zsh, git, nvim) or 'self' to edit config.toml")]
    command: Option<String>,

    #[arg(help = "Module within the command (e.g., aliases, exports)")]
    module: Option<String>,
}

fn open_in_editor(editor: &str, path: &std::path::Path) {
    let status = Command::new(editor)
        .arg(path)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to launch editor '{}': {}", editor, e);
            std::process::exit(1);
        });
    if !status.success() {
        eprintln!("Editor '{}' exited with error", editor);
        std::process::exit(1);
    }
}

fn main() {
    let cli = Cli::parse();
    let cfg = Config::load();
    let editor = cfg.editor().to_string();

    if cli.list {
        println!("Available config commands:");
        for (name, cmd) in cfg.list_commands() {
            let desc = cmd.description.as_deref().unwrap_or("");
            println!("  {:<12} {}", name, desc);
            if let Some(modules) = &cmd.modules {
                for m in modules {
                    let d = m.desc.as_deref().unwrap_or("");
                    println!("  {:<6}{:<6} {}", "", m.name, d);
                }
            }
        }
        return;
    }

    let cmd_name = match &cli.command {
        Some(name) => name,
        None => {
            println!("Usage: dot <command> [module]");
            println!("Run 'dot --list' to see available commands.");
            std::process::exit(1);
        }
    };

    if cmd_name == "self" {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let config_path = std::path::PathBuf::from(home)
            .join(".config")
            .join("dot")
            .join("config.toml");
        open_in_editor(&editor, &config_path);
        return;
    }

    let cmd = match cfg.find_command(cmd_name) {
        Some(c) => c,
        None => {
            eprintln!("Unknown command: {}", cmd_name);
            eprintln!("Run 'dot --list' to see available commands.");
            std::process::exit(1);
        }
    };

    // If a module is specified, look for it
    if let Some(module_name) = &cli.module {
        if let Some(modules) = &cmd.modules {
            if let Some(module) = modules.iter().find(|m| m.name == *module_name) {
                let path = Config::resolve_path(&module.path);
                open_in_editor(&editor, &path);
                return;
            }
        }
        eprintln!("Unknown module '{}' for command '{}'", module_name, cmd_name);
        if let Some(modules) = &cmd.modules {
            eprintln!("Available modules: {}", modules.iter().map(|m| &m.name).cloned().collect::<Vec<_>>().join(", "));
        }
        std::process::exit(1);
    }

    // Open the main command path
    let path = Config::resolve_path(&cmd.path);
    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        std::process::exit(1);
    }
    open_in_editor(&editor, &path);
}
