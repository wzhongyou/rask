mod cli;
mod input;
mod repl;

use clap::Parser;
use cli::{Cli, Command, ConfigAction};
use rask_core::{config::Config, paths::config_path};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    if let Some(Command::Config { action }) = args.command {
        let mut config = Config::load()?;
        match action {
            ConfigAction::Set { key, value } => {
                config.set(&key, &value)?;
                config.save()?;
                println!("saved: {key} = {value}");
            }
            ConfigAction::Show => {
                let path = config_path().unwrap_or_default();
                println!("# {}\n{}", path.display(), toml::to_string_pretty(&config).unwrap());
            }
        }
        return Ok(());
    }

    let config = Config::load()?;
    let model = args.model.as_deref().unwrap_or(config.default_model()).to_string();

    match args.prompt {
        Some(prompt) => repl::once(&prompt, &model, &config).await,
        None => repl::run(&model, &config).await,
    }
}
