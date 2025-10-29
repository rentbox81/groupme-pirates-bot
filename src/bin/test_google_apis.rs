use groupme_bot::{config::Config, google_client::GoogleClient};
use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Google APIs Diagnostic Tool");
    println!("================================\n");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = match Config::from_env() {
        Ok(config) => {
            println!("✅ Configuration loaded successfully");
            config
        }
        Err(e) => {
            println!("❌ Failed to load configuration: {}", e);
            return Ok(());
        }
    };

    println!("📋 Current Configuration:");
    println!("  Google API Key: {}...{}", 
             &config.google_api_key[..10], 
             &config.google_api_key[config.google_api_key.len()-4..]);
    println!("  Sheet ID: {}", config.sheet_id);
    println!("  Calendar Webcal URL: {:?}\n", config.calendar_webcal_url);

    // Test API key validity (only for Sheets, since we use webcal for calendar)
    println!("🔑 Testing API Key validity with Sheets API...");
    match test_sheets_access(&config).await {
        Ok(_) => println!("✅ API Key works with Sheets API\n"),
        Err(e) => {
            println!("❌ API Key test failed: {}", e);
            print_api_key_help();
            return Ok(());
        }
    }

    // Test Calendar API access
    println!("📅 Testing Calendar API access...");
    match test_calendar_access(&config).await {
        Ok(calendar_info) => {
            println!("✅ Calendar API access successful");
            println!("📋 Calendar Info: {}", calendar_info);
        }
        Err(e) => {
            println!("❌ Calendar API access failed: {}", e);
            if let Some(url) = &config.calendar_webcal_url { print_calendar_help(url); }
        }
    }

    // Test Sheets API access
    println!("\n📊 Testing Sheets API access...");
    match test_sheets_access(&config).await {
        Ok(sheets_info) => {
            println!("✅ Sheets API access successful");
            println!("📋 Sheet Info: {}", sheets_info);
        }
        Err(e) => {
            println!("❌ Sheets API access failed: {}", e);
            print_sheets_help(&config.sheet_id);
        }
    }

    println!("\n🎯 Testing with GoogleClient...");
    let google_client = GoogleClient::new(config);
    
    match google_client.get_calendar_events().await {
        Ok(events) => {
            println!("✅ Calendar events retrieved: {} events", events.len());
            for (i, (date, title)) in events.iter().take(5).enumerate() {
                println!("  {}. {} - {}", i+1, date, title);
            }
            if events.len() > 5 {
                println!("  ... and {} more events", events.len() - 5);
            }
        }
        Err(e) => {
            println!("❌ Failed to get calendar events: {}", e);
        }
    }

    match google_client.get_sheets_data().await {
        Ok(data) => {
            println!("✅ Sheets data retrieved: {} rows", data.len());
            for (i, (date, title, location, snacks, _livestream, _scoreboard, _pitch_count, home_team)) in data.iter().take(3).enumerate() {
                let snacks_display = if snacks.trim().is_empty() { "NEEDED" } else { snacks };
                println!("  {}. {} - {} at {} ({}) - Snacks: {}", i+1, date, title, location, home_team, snacks_display);
            }
            if data.len() > 3 {
                println!("  ... and {} more rows", data.len() - 3);
            }
        }
        Err(e) => {
            println!("❌ Failed to get sheets data: {}", e);
        }
    }

    Ok(())
}


async fn test_calendar_access(config: &Config) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Test webcal URL access
    let https_url = config.calendar_webcal_url.as_ref().unwrap().replace("webcal://", "https://");
    
    let response = client.get(&https_url).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Webcal URL request failed ({}): {}", status, error_text).into());
    }
    
    let ical_text = response.text().await?;
    let line_count = ical_text.lines().count();
    let has_vcalendar = ical_text.contains("BEGIN:VCALENDAR");
    let has_events = ical_text.contains("BEGIN:VEVENT");
    
    Ok(format!("iCal data: {} lines, has calendar: {}, has events: {}", line_count, has_vcalendar, has_events))
}

async fn test_sheets_access(config: &Config) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Get spreadsheet info
    let sheets_url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}?key={}",
        &config.sheet_id,
        &config.google_api_key
    );
    
    let response = client.get(&sheets_url).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Sheets info request failed ({}): {}", status, error_text).into());
    }
    
    let sheets_info: Value = response.json().await?;
    let title = sheets_info["properties"]["title"].as_str().unwrap_or("Unknown");
    
    Ok(format!("Title: '{}'", title))
}

fn print_api_key_help() {
    println!("\n🆘 API Key Help:");
    println!("1. Go to Google Cloud Console: https://console.cloud.google.com/");
    println!("2. Enable the Google Sheets API (Calendar API not needed - we use webcal URLs)");
    println!("3. Create an API Key in 'Credentials'");
    println!("4. Make sure the API key has access to the Sheets API");
    println!("5. Ensure your Google Sheet is publicly viewable (Anyone with link can view)");
}

fn print_calendar_help(webcal_url: &str) {
    println!("\n🆘 Calendar Access Help:");
    println!("Webcal URL: {}", webcal_url);
    
    if webcal_url.starts_with("webcal://") {
        println!("📝 Webcal URL troubleshooting:");
        println!("1. Make sure the calendar is publicly accessible");
        println!("2. Test the https:// version of the URL in a browser:");
        println!("   {}", webcal_url.replace("webcal://", "https://"));
        println!("3. For Google Calendar:");
        println!("   - Go to Calendar Settings → Your calendar → Settings and sharing");
        println!("   - Make calendar public");
        println!("   - Copy the 'Public address in iCal format' URL");
        println!("   - Replace 'https://' with 'webcal://' in the URL");
    } else {
        println!("⚠️  URL should start with 'webcal://' not 'https://'");
        println!("📝 Please use the webcal:// format for the URL");
    }
}

fn print_sheets_help(sheet_id: &str) {
    println!("\n🆘 Sheets Access Help:");
    println!("Sheet ID: {}", sheet_id);
    println!("📝 For Google Sheets access:");
    println!("1. Make sure the spreadsheet is shared publicly (Anyone with link can view)");
    println!("2. Or share it with the service account email if using OAuth");
    println!("3. The Sheet ID is the long string in the URL:");
    println!("   https://docs.google.com/spreadsheets/d/SHEET_ID_HERE/edit");
}
