mod config;

use clap::Parser;
use clap::CommandFactory;
use clap_complete::{Shell, generate};
use config::Config;
use std::io;
use std::process::Command;

#[derive(Parser)]
#[command(name = "dot", version, about = "Quickly open config files")]
struct Cli {
    #[arg(short = 'l', long = "list", help = "List all available commands")]
    list: bool,

    #[command(subcommand)]
    subcommand: Option<Subcommand>,

    #[arg(help = "Config command (e.g., zsh, git, nvim) or 'self' to edit config.toml")]
    command: Option<String>,

    #[arg(help = "Module within the command (e.g., aliases, exports)")]
    module: Option<String>,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// Generate shell completion script
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
    /// [internal] List command names for shell completion
    #[command(hide = true)]
    CompleteCommands,
    /// [internal] List module names for a command
    #[command(hide = true)]
    CompleteModules {
        command: String,
    },
}

fn print_complete_commands(cfg: &Config) {
    for (name, cmd) in cfg.list_commands() {
        println!("{}:{}", name, cmd.description.as_deref().unwrap_or(""));
    }
}

fn print_complete_modules(cfg: &Config, cmd_name: &str) {
    if let Some(cmd) = cfg.find_command(cmd_name) {
        if let Some(modules) = &cmd.modules {
            for m in modules {
                println!("{}:{}", m.name, m.desc.as_deref().unwrap_or(""));
            }
        }
    }
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

    if let Some(Subcommand::Completions { shell }) = &cli.subcommand {
        let mut cmd = Cli::command();
        generate(*shell, &mut cmd, "dot", &mut io::stdout());
        return;
    }

    if let Some(Subcommand::CompleteCommands) = &cli.subcommand {
        print_complete_commands(&cfg);
        return;
    }

    if let Some(Subcommand::CompleteModules { command }) = &cli.subcommand {
        print_complete_modules(&cfg, command);
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
