use groupme_bot::{config::Config, google_client::GoogleClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Detailed Google Sheets Data Analysis");
    println!("=====================================\n");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            println!("❌ Failed to load configuration: {}", e);
            return Ok(());
        }
    };

    println!("🎯 Testing with GoogleClient...");
    let google_client = GoogleClient::new(config);

    match google_client.get_sheets_data().await {
        Ok(data) => {
            println!("✅ Sheets data retrieved: {} rows\n", data.len());
            println!("Column mapping:");
            println!("  A = Date, B = Time, C = Location, D = Home Team, E = Snacks, F = Livestream, G = Scoreboard, H = Pitch Count\n");
            
            for (i, (date, time, location, home_team, snacks, livestream, scoreboard, pitch_count)) in data.iter().enumerate() {
                if date.to_string().contains("2025-08-27") {
                    println!("🎯 FOUND 2025-08-27 EVENT:");
                    println!("  Date (A): {}", date);
                    println!("  Time (B): '{}'", time);
                    println!("  Location (C): '{}'", location);
                    println!("  Home Team (D): '{}'", home_team);
                    println!("  Snacks (E): '{}'", snacks);
                    println!("  Livestream (F): '{}'", livestream);
                    println!("  Scoreboard (G): '{}'", scoreboard);
                    println!("  Pitch Count (H): '{}'", pitch_count);
                    println!();
                }
                
                if i < 3 {
                    println!("Row {}: {} | {} | {} | {} | {} | {} | {} | {}", 
                        i+1, date, time, location, home_team, snacks, livestream, scoreboard, pitch_count);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get sheets data: {}", e);
        }
    }

    Ok(())
}
