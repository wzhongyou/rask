use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rask", about = "Terminal AI assistant")]
pub struct Cli {
    /// Prompt for one-shot mode; omit to enter REPL
    pub prompt: Option<String>,

    /// Model to use (overrides config)
    #[arg(short, long)]
    pub model: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a config value: rask config set <key> <value>
    Set { key: String, value: String },
    /// Show current config
    Show,
}
