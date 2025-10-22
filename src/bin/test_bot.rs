use groupme_bot::{config::Config, service::BotService, parser::CommandParser};
use std::io::{self, Write};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = match Config::from_env() {
        Ok(config) => {
            info!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            eprintln!("Error: Failed to load configuration: {}", e);
            eprintln!("Make sure your .env file is properly configured.");
            std::process::exit(1);
        }
    };

    // Create services
    let bot_service = BotService::new(config.clone());
    let command_parser = CommandParser::new(config.groupme_bot_name.clone());

    println!("🤖 GroupMe Bot CLI Tester");
    println!("Bot Name: {}", config.groupme_bot_name);
    println!("Enter messages as if you're typing in GroupMe chat.");
    println!("Type 'quit' or 'exit' to stop testing.\n");

    loop {
        print!("Enter message: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!("Goodbye! 👋");
            break;
        }

        // Parse the command
        match command_parser.parse_message(input, None, None).await {
            Ok(Some(command)) => {
                println!("📝 Parsed command: {:?}", command);
                
                // Handle the command
                match bot_service.handle_command(command, None).await {
                    Ok(response) => {
                        println!("🤖 Bot Response:");
                        println!("─────────────────");
                        println!("{}", response);
                        println!("─────────────────\n");
                    }
                    Err(e) => {
                        error!("Failed to handle command: {}", e);
                        println!("❌ Error: {}\n", e);
                    }
                }
            }
            Ok(None) => {
                println!("ℹ️  Message not directed at bot or empty\n");
            }
            Err(e) => {
                error!("Failed to parse command: {}", e);
                println!("❌ Parse Error: {}\n", e);
            }
        }
    }

    Ok(())
}
