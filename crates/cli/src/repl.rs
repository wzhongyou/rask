use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rask_core::{
    client::{AiClient, infer_provider, openai::OpenAiClient, anthropic::AnthropicClient},
    config::Config,
    history::{HistoryEntry, save as save_history},
    session::Session,
};
use std::time::Duration;

const COMMANDS: &[(&str, &str)] = &[
    ("/help",    "show available commands"),
    ("/model",   "switch model: /model <name>"),
    ("/clear",   "clear conversation context"),
];

pub async fn run(model: &str, config: &Config) -> Result<()> {
    let mut session = Session::new(model);
    let client = build_client(model, config)?;
    let mut history: Vec<String> = Vec::new();

    println!("rask — type /help for commands, exit or Ctrl+D to quit\n");

    loop {
        match crate::input::read_line("rask> ", &mut history) {
            Ok(Some(line)) => {
                let input = line.trim().to_string();
                if input.is_empty() { continue; }
                if input == "exit" { break; }
                history.push(input.clone());

                if let Some(rest) = input.strip_prefix('/') {
                    handle_command(rest, &mut session);
                    continue;
                }

                session.push_user(&input);
                let spinner = ProgressBar::new_spinner();
                spinner.set_style(ProgressStyle::default_spinner()
                    .template("{spinner}").unwrap());
                spinner.enable_steady_tick(Duration::from_millis(80));

                match client.chat(&session.messages).await {
                    Ok(reply) => {
                        spinner.finish_and_clear();
                        println!("\n{}\n", reply);
                        session.push_assistant(reply);
                    }
                    Err(e) => {
                        spinner.finish_and_clear();
                        eprintln!("error: {e}");
                    }
                }
            }
            Ok(None) => break,
            Err(e) => { eprintln!("error: {e}"); break; }
        }
    }

    // persist session if there was any user interaction
    let user_count = session.messages.iter().filter(|m| m.role == "user").count();
    if user_count > 0 {
        let entry = HistoryEntry::new(session.model.clone(), session.messages.clone());
        let _ = save_history(&entry);
    }

    Ok(())
}

fn handle_command(cmd: &str, session: &mut Session) {
    let mut parts = cmd.splitn(2, ' ');
    match parts.next().unwrap_or("") {
        "help" => {
            for (name, desc) in COMMANDS {
                println!("  {:<12} {}", name, desc);
            }
            println!("  {:<12} {}", "exit", "quit rask");
            println!();
        }
        "clear" => {
            *session = Session::new(&session.model.clone());
            println!("context cleared\n");
        }
        "model" => {
            if let Some(name) = parts.next() {
                *session = Session::new(name);
                println!("model switched to {name}\n");
            } else {
                println!("current model: {}\n", session.model);
            }
        }
        other => eprintln!("unknown command: /{other}  (try /help)\n"),
    }
}

pub async fn once(prompt: &str, model: &str, config: &Config) -> Result<()> {
    let mut session = Session::new(model);
    let client = build_client(model, config)?;
    session.push_user(prompt);
    let reply = client.chat(&session.messages).await?;
    println!("{reply}");
    Ok(())
}

fn build_client(model: &str, config: &Config) -> Result<Box<dyn AiClient>> {
    let provider = infer_provider(model);
    let pcfg = config.provider(provider)
        .ok_or_else(|| anyhow::anyhow!(
            "no API key for provider '{provider}'\nrun: rask config set providers.{provider}.api_key <key>"
        ))?;

    let client: Box<dyn AiClient> = match provider {
        "anthropic" => Box::new(AnthropicClient::new(&pcfg.api_key, model)),
        _ => {
            let base_url = pcfg.base_url.as_deref().unwrap_or("https://api.openai.com/v1");
            Box::new(OpenAiClient::new(&pcfg.api_key, base_url, model))
        }
    };
    Ok(client)
}
