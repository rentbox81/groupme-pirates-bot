use groupme_bot::{parser::CommandParser, models::BotCommand};
use std::io::{self, Write};
use chrono::NaiveDate;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    let bot_name = std::env::var("GROUPME_BOT_NAME").unwrap_or_else(|_| "TestBot".to_string());
    let command_parser = CommandParser::new(bot_name.clone());

    println!("🤖 GroupMe Bot CLI Tester (Mock Mode)");
    println!("Bot Name: {}", bot_name);
    println!("This mode uses mock data and doesn't require real API credentials.");
    println!("Enter messages as if you're typing in GroupMe chat.");
    println!("Type 'quit' or 'exit' to stop testing.\n");

    println!("💡 Example commands to try:");
    println!("  @{} commands", bot_name);
    println!("  @{} next game", bot_name);
    println!("  @{} next 3 games", bot_name);
    println!("  @{} next game snacks", bot_name);
    println!("  @{} lets go pirates\n", bot_name);

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
        match command_parser.parse_message(input, None, None, &[]).await {
            Ok(Some(command)) => {
                println!("📝 Parsed command: {:?}", command);
                
                // Handle the command with mock data
                let response = handle_command_mock(command, &bot_name);
                
                println!("🤖 Bot Response:");
                println!("─────────────────");
                println!("{}", response);
                println!("─────────────────\n");
            }
            Ok(None) => {
                println!("ℹ️  Message not directed at bot or empty\n");
            }
            Err(e) => {
                println!("❌ Parse Error: {}\n", e);
            }
        }
    }

    Ok(())
}

fn handle_command_mock(command: BotCommand, bot_name: &str) -> String {
    match command {
        BotCommand::Commands => {
            format!(
                "⚾ {} Commands:\\n\\n\
                 🏴‍☠️ Game Info:\\n\
                 • @{} next game - Full details for next game\\n\
                 • @{} next 3 games - Show next 3 games\\n\
                 • @{} next game snacks - Get snacks info for next game\\n\\n\
                 🏴‍☠️ Team Spirit:\\n\
                 • @{} lets go pirates - Get a Pirates fact!\\n\\n\
                 🏴‍☠️ Volunteers:\\n\
                 • @{} volunteer snacks 2025-01-15 John - Sign up to volunteer\\n\
                 • @{} volunteers - Show all volunteer needs\\n\
                 📋 Categories: time, location, home, snacks, livestream, scoreboard, pitchcount",
                bot_name, bot_name, bot_name, bot_name, bot_name, bot_name, bot_name
            )
        }
        BotCommand::NextGame => {
            "🏴‍☠️ Next Game: 7:30pm - Away\\nDate: 2024-01-15\\nTime: 7:30pm\\nLocation: Memorial Stadium (https://maps.google.com/?q=Memorial%20Stadium)\\nHome Team: Away\\nSnacks: Sarah Johnson\\nLivestream: Mike Wilson\\nScoreboard: Jennifer Smith\\nPitch Count: David Brown".to_string()
        }
        BotCommand::NextGames(count) => {
            let mut response = format!("🏴‍☠️ Next {} Games:\\n\\n", count);
            let locations = [
                "Memorial Stadium (https://maps.google.com/?q=Memorial%20Stadium)",
                "Central Park Field (https://maps.google.com/?q=Central%20Park%20Field)",
                "Riverside Complex (https://maps.google.com/?q=Riverside%20Complex)"
            ];
            for i in 0..count.min(3) {
                response.push_str(&format!(
                    "📅 2024-01-{:02} - {}\\n⏰ Time: 7:30pm\\n📍 Location: {}\\n🏠 Home Team: {}\\n\\n",
                    15 + i * 7,
                    ["Pirates vs Cardinals", "Pirates vs Cubs", "Pirates vs Reds"][i],
                    locations[i],
                    ["Away", "Home", "Away"][i]
                ));
            }
            response
        }
        BotCommand::NextGameCategory(category) => {
            match category.as_str() {
                "location" => "⚾ Next game location: Memorial Stadium (https://maps.google.com/?q=Memorial%20Stadium)".to_string(),
                "snacks" => "⚾ Next game snacks: Sarah Johnson".to_string(),
                "livestream" => "⚾ Next game livestream: Mike Wilson".to_string(),
                "scoreboard" => "⚾ Next game scoreboard: Jennifer Smith".to_string(),
                "pitchcount" => "⚾ Next game pitchcount: David Brown".to_string(),
                "time" => "⚾ Next game time: 7:30pm".to_string(),
                "home" => "⚾ Next game home: Away".to_string(),
                _ => format!("❌ No {} information available for the next game.", category),
            }
        }
        BotCommand::LetsGo(team) => {
            match team.as_str() {
                "pirates" => "⚾ The Pittsburgh Pirates won the first World Series ever played in 1903, defeating the Boston Red Sox!\\n\\n🏴‍☠️ Raise the Jolly Roger! ⚾".to_string(),
                _ => "⚾ Great team spirit! Here's a Pirates fact for you: Roberto Clemente was the first Latino player inducted into the Baseball Hall of Fame!\\n\\n🏴‍☠️ Ahoy matey! ⚾".to_string(),
            }
        }
        BotCommand::Volunteer(date, role, person) => {
            format!("✅ {} has been assigned to {} for {} (Mock Game)!", person, role, date)
        }
        BotCommand::VolunteerNextGame(role, person) => {
            format!("✅ {} has been assigned to {} for the next game (Mock)!", person, role)
        }
        BotCommand::ShowVolunteers(maybe_date) => {
            if let Some(date) = maybe_date {
                format!(
                    "🏴‍☠️ Volunteer status for {} (Mock Game):\n\n\
                     Date: {}\\nTime: 7:30pm\\nLocation: Memorial Stadium\\nHome Team: Away\\n\
                     Snacks: ⚠️ NEEDED\\nLivestream: Mike Wilson\\nScoreboard: ⚠️ NEEDED\\nPitch Count: David Brown\\n\\n\
                     ⚠️ Still needed: snacks, scoreboard",
                    date, date
                )
            } else {
                "🏴‍☠️ Volunteer status for upcoming events:\n\n\
                 2024-01-15 (Mock Game):\n⚠️ Still needed: snacks, scoreboard\n\n\
                 2024-01-22 (Mock Game):\n⚠️ Still needed: livestream, pitchcount".to_string()
            }
        }
        BotCommand::RemoveVolunteer(person, role, date) => {
            let date_str = date.map(|d| d.to_string()).unwrap_or_else(|| "next game".to_string());
            format!("✅ {} has been removed from {} for {} (Mock)!", person, role, date_str)
        }
        BotCommand::AssignVolunteer(person, role, date) => {
            let date_str = date.map(|d| d.to_string()).unwrap_or_else(|| "next game".to_string());
            format!("✅ {} has been assigned to {} for {} by admin (Mock)!", person, role, date_str)
        }
        BotCommand::AddModerator(user_id) => {
            format!("✅ Added moderator: {} (Mock)!", user_id)
        }
        BotCommand::RemoveModerator(user_id) => {
            format!("✅ Removed moderator: {} (Mock)!", user_id)
        }
        BotCommand::ListModerators => {
            "🏴‍☠️ Moderators (Mock):\n- user123\n- user456\n\nAdmin: admin_user".to_string()
        }
        BotCommand::ListBotMessages(count) => {
            format!("🏴‍☠️ Recent bot messages (Mock - last {}):\n\n1. ID: 12345678901234 - ⚾ Next Game: Pirates vs Cardinals...\n2. ID: 12345678901235 - ✅ John has been assigned to snacks...\n\n💡 Note: Messages can only be deleted manually through the GroupMe mobile app.", count)
        }
    }
}

fn get_mock_events() -> std::collections::HashMap<NaiveDate, (String, String, String, String, String)> {
    let mut events = std::collections::HashMap::new();
    
    // Add some mock events
    events.insert(
        NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        ("Memorial Stadium".to_string(), "Sarah Johnson".to_string(), "Mike Wilson".to_string(), "Jennifer Smith".to_string(), "David Brown".to_string())
    );
    
    events.insert(
        NaiveDate::from_ymd_opt(2024, 1, 22).unwrap(),
        ("Central Park Field".to_string(), "Tom Anderson".to_string(), "Lisa Davis".to_string(), "Robert Taylor".to_string(), "Emma Martinez".to_string())
    );
    
    events.insert(
        NaiveDate::from_ymd_opt(2024, 1, 29).unwrap(),
        ("Riverside Complex".to_string(), "John Miller".to_string(), "Amy Garcia".to_string(), "Chris Lee".to_string(), "Maria Rodriguez".to_string())
    );
    
    events
}
