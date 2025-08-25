use chrono::{NaiveDate, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

use crate::config::Config;
use crate::error::Result;
use crate::google_client::GoogleClient;
use crate::groupme_client::GroupMeClient;
use crate::models::{CorrelatedEvent, EventData, BotCommand};
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Clone)]
pub struct BotService {
    google_client: GoogleClient,
    groupme_client: GroupMeClient,
    config: Config,
    // Cache for event data to reduce API calls and enable volunteer modifications
    event_cache: Arc<RwLock<HashMap<NaiveDate, CorrelatedEvent>>>,
}

impl BotService {
    pub fn new(config: Config) -> Self {
        let google_client = GoogleClient::new(config.clone());
        let groupme_client = GroupMeClient::new(config.clone());
        
        Self {
            google_client,
            groupme_client,
            config,
            event_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn correlate_data(&self) -> Result<HashMap<NaiveDate, CorrelatedEvent>> {
        info!("Starting data correlation");
        
        let calendar_events = self.google_client.get_calendar_events().await?;
        let sheets_data = self.google_client.get_sheets_data().await?;
        
        let mut correlated_map = HashMap::new();
        
        // Start with calendar events as the source of truth
        for (date, summary) in calendar_events {
            // Create placeholder EventData for events without sheet data
            let placeholder_data = EventData::new(
                date,
                "TBD".to_string(),
                "TBD".to_string(),
                "TBD".to_string(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            );
            
            correlated_map.insert(
                date,
                CorrelatedEvent {
                    event_date: date,
                    event_summary: summary,
                    data: placeholder_data,
                },
            );
        }
        
        // Augment with data from Google Sheets
        for (date, time, location, snacks, livestream, scoreboard, pitch_count, home_team) in sheets_data {
            if let Some(event) = correlated_map.get_mut(&date) {
                // Always update with sheet data (sheet is more detailed than calendar)
                event.data = EventData::new(
                    date,
                    time.clone(),
                    location,
                    home_team.clone(),
                    snacks,
                    livestream,
                    scoreboard,
                    pitch_count,
                );
                
                // Update the event summary to be more descriptive using sheet data
                if !time.is_empty() && !home_team.is_empty() {
                    event.event_summary = format!("{} vs. {}", time, home_team);
                }
                
                info!("Updated event {} with sheet data", date);
            } else {
                // Even if no calendar event, create one from sheet data
                info!("Creating event from sheet data for {} (no matching calendar event)", date);
                let event_data = EventData::new(
                    date,
                    time.clone(),
                    location,
                    home_team.clone(),
                    snacks,
                    livestream,
                    scoreboard,
                    pitch_count,
                );
                
                let summary = if !time.is_empty() && !home_team.is_empty() {
                    format!("{} vs. {}", time, home_team)
                } else {
                    format!("Event on {}", date)
                };
                
                correlated_map.insert(
                    date,
                    CorrelatedEvent {
                        event_date: date,
                        event_summary: summary,
                        data: event_data,
                    },
                );
            }
        }
        
        info!("Correlation complete: {} events with data", correlated_map.len());
        
        // Update cache with fresh data
        if let Ok(mut cache) = self.event_cache.write() {
            cache.clear();
            cache.extend(correlated_map.clone());
        }
        
        Ok(correlated_map)
    }
    
    pub async fn get_cached_or_fresh_data(&self) -> Result<HashMap<NaiveDate, CorrelatedEvent>> {
        // Check if cache is populated
        if let Ok(cache) = self.event_cache.read() {
            if !cache.is_empty() {
                return Ok(cache.clone());
            }
        }
        
        // Cache is empty, correlate fresh data
        self.correlate_data().await
    }

    pub async fn find_next_event(&self) -> Result<Option<CorrelatedEvent>> {
        let events = self.correlate_data().await?;
        let today = Utc::now().date_naive();
        
        let mut next_event: Option<CorrelatedEvent> = None;
        let mut min_diff_days = i64::MAX;

        for (date, event) in events.iter() {
            let diff = (*date - today).num_days();
            if diff >= 0 && diff < min_diff_days {
                min_diff_days = diff;
                next_event = Some(event.clone());
            }
        }
        
        Ok(next_event)
    }

    pub async fn find_event_by_date(&self, query_date: NaiveDate) -> Result<Option<CorrelatedEvent>> {
        // First check cache
        if let Ok(cache) = self.event_cache.read() {
            if let Some(event) = cache.get(&query_date) {
                return Ok(Some(event.clone()));
            }
        }
        
        // Not in cache, get fresh data
        let events = self.correlate_data().await?;
        Ok(events.get(&query_date).cloned())
    }

    pub async fn handle_command(&self, command: BotCommand) -> Result<String> {
        match command {
            BotCommand::NextGame => {
                // @bot next game
                match self.find_next_event().await? {
                    Some(event) => {
                        let mut response = format!("🏴‍☠️ Next Game: {}
", event.event_summary);
                        response.push_str(&event.data.format_all());
                        Ok(response)
                    }
                    None => Ok("⚾ No upcoming games found.".to_string()),
                }
            }
            
            BotCommand::NextGames(count) => {
                // @bot next X games
                let events = self.correlate_data().await?;
                let today = Utc::now().date_naive();
                
                let mut upcoming_events: Vec<_> = events.iter()
                    .filter(|(date, _)| **date >= today)
                    .collect();
                
                upcoming_events.sort_by_key(|(date, _)| *date);
                
                if upcoming_events.is_empty() {
                    return Ok("⚾ No upcoming games found.".to_string());
                }
                
                let mut response = format!("🏴‍☠️ Next {} Games:

", count.min(upcoming_events.len()));
                
                for (date, event) in upcoming_events.iter().take(count) {
                    response.push_str(&format!("📅 {} - {}
", date.format("%Y-%m-%d"), event.event_summary));
                    response.push_str(&format!("⏰ Time: {}
", event.data.time));
                    response.push_str(&format!("📍 Location: {}
", event.data.format_location_with_link()));
                    response.push_str(&format!("🏠 Home Team: {}

", event.data.home_team));
                }
                
                Ok(response)
            }
            
            BotCommand::NextGameCategory(category) => {
                // @bot next game snacks
                match self.find_next_event().await? {
                    Some(event) => {
                        match category.to_lowercase().as_str() {
                            "location" => {
                                Ok(format!("⚾ Next game location: {}", event.data.format_location_with_link()))
                            }
                            _ => {
                                if let Some(data) = event.data.get_field(&category) {
                                    Ok(format!("⚾ Next game {}: {}", category, data))
                                } else {
                                    Ok(format!("❌ No {} information available for the next game.", category))
                                }
                            }
                        }
                    }
                    None => Ok("⚾ No upcoming games found.".to_string()),
                }
            }
            
            BotCommand::LetsGo(team) => {
                // @bot lets go pirates
                Ok(PiratesFacts::get_team_fact(&team))
            }
            
            BotCommand::Volunteer(date, role, person) => {
                self.handle_volunteer_assignment(date, role, person).await
            }
            
            BotCommand::ShowVolunteers(maybe_date) => {
                self.handle_show_volunteers(maybe_date).await
            }
            
            BotCommand::Commands => {
                Ok(format!(
                    "⚾ {} Commands:

\
                     🏴‍☠️ Game Info:
\
                     • @{} next game - Full details for next game
\
                     • @{} next 3 games - Show next 3 games
\
                     • @{} next game snacks - Get snacks info for next game

\
                     🏴‍☠️ Team Spirit:
\
                     • @{} lets go pirates - Get a Pirates fact!

\
                     🏴‍☠️ Volunteers:
\
                     • @{} volunteer snacks 2025-01-15 John - Sign up to volunteer
\
                     • @{} volunteers - Show all volunteer needs
\
                     • @{} volunteers 2025-01-15 - Show needs for specific date

\
                     📋 Categories: time, location, home, snacks, livestream, scoreboard, pitchcount
\
                     🏴‍☠️ Raise the Jolly Roger! ⚾",
                    self.config.groupme_bot_name,
                    self.config.groupme_bot_name,
                    self.config.groupme_bot_name,
                    self.config.groupme_bot_name,
                    self.config.groupme_bot_name,
                    self.config.groupme_bot_name,
                    self.config.groupme_bot_name,
                    self.config.groupme_bot_name
                ))
            }
        }
    }

    pub async fn send_response(&self, message: &str) -> Result<()> {
        self.groupme_client.send_message(message).await
    }
    
    async fn handle_volunteer_assignment(&self, date: NaiveDate, role: String, person: String) -> Result<String> {
        match self.find_event_by_date(date).await? {
            Some(mut event) => {
                if event.data.is_role_available(&role) {
                    // Update Google Sheets first
                    match self.google_client.update_volunteer_assignment(date, &role, &person).await {
                        Ok(_) => {
                            // If sheet update successful, also update local cache
                            if event.data.assign_volunteer(&role, &person) {
                                // Update cache
                                if let Ok(mut cache) = self.event_cache.write() {
                                    cache.insert(date, event.clone());
                                }
                                
                                Ok(format!("✅ {} has been assigned to {} for {} ({})!", 
                                         person, role, date, event.event_summary))
                            } else {
                                Ok("❌ Assignment failed. Code: VOL002".to_string())
                            }
                        }
                        Err(e) => {
                            warn!("Failed to update Google Sheet: {}", e);
                            Ok("❌ Update failed. Code: VOL001".to_string())
                        }
                    }
                } else {
                    let current_volunteer = match role.to_lowercase().as_str() {
                        "snacks" => event.data.snacks.as_ref(),
                        "livestream" => event.data.livestream.as_ref(),
                        "scoreboard" => event.data.scoreboard.as_ref(),
                        "pitchcount" | "pitch_count" => event.data.pitch_count.as_ref(),
                        _ => None,
                    };
                    
                    if let Some(current) = current_volunteer {
                        Ok(format!("❌ {} is already assigned to {} for {} ({}).", 
                                 current, role, date, event.event_summary))
                    } else {
                        Ok("❌ Assignment failed. Code: VOL003".to_string())
                    }
                }
            }
            None => Ok(format!("❌ No event found for {}.", date)),
        }
    }
    
    async fn handle_show_volunteers(&self, maybe_date: Option<NaiveDate>) -> Result<String> {
        match maybe_date {
            Some(date) => {
                match self.find_event_by_date(date).await? {
                    Some(event) => {
                        let mut response = format!("🏴‍☠️ Volunteer status for {} ({}):\n\n", 
                                                  date, event.event_summary);
                        response.push_str(&event.data.format_all());
                        response.push_str(&format!("\n{}", event.data.format_volunteer_needs()));
                        Ok(response)
                    }
                    None => Ok(format!("❌ No event found for {}.", date)),
                }
            }
            None => {
                // Show volunteer status for all upcoming events
                let events = self.correlate_data().await?;
                let today = Utc::now().date_naive();
                
                let mut upcoming_events: Vec<_> = events.iter()
                    .filter(|(date, _)| **date >= today)
                    .collect();
                
                upcoming_events.sort_by_key(|(date, _)| *date);
                
                if upcoming_events.is_empty() {
                    Ok("❌ No upcoming events found.".to_string())
                } else {
                    let mut response = "🏴‍☠️ Volunteer status for upcoming events:\n\n".to_string();
                    
                    for (date, event) in upcoming_events.iter().take(5) {
                        response.push_str(&format!("{} ({}):\n", date, event.event_summary));
                        response.push_str(&format!("{}\n", event.data.format_volunteer_needs()));
                        response.push('\n');
                    }
                    
                    if upcoming_events.len() > 5 {
                        response.push_str(&format!("... and {} more events", upcoming_events.len() - 5));
                    }
                    
                    Ok(response)
                }
            }
        }
    }
}

// Inline Pirates facts to avoid module import issues
struct PiratesFacts;

impl PiratesFacts {
    fn get_team_fact(team_name: &str) -> String {
        match team_name.to_lowercase().as_str() {
            "pirates" => {
                let facts = [
                    "🏴‍☠️ The Pittsburgh Pirates were the first professional sports team to win a championship via walk-off home run in 1960!",
                    "⚾ The Pirates were the first MLB team to field an all-minority starting lineup on September 1, 1971!",
                    "🏴‍☠️ Roberto Clemente was the first Latino player to reach 3,000 hits and was inducted into the Baseball Hall of Fame in 1973!",
                    "⚾ Three Rivers Stadium was home to the Pirates from 1970-2000 and hosted the 1979 World Series championship!",
                    "🏴‍☠️ The Pirates' 'We Are Family' team of 1979 came back from a 3-1 deficit to win the World Series!",
                    "⚾ PNC Park opened in 2001 and is consistently ranked as one of the most beautiful ballparks in baseball!",
                    "🏴‍☠️ Honus Wagner, the 'Flying Dutchman', played shortstop for the Pirates and led them to their first World Series title in 1909!",
                    "⚾ The Pirates were founded in 1881, making them one of the oldest franchises in Major League Baseball!",
                    "🏴‍☠️ The team is called 'Pirates' because they 'pirated' a player from another team in 1891!",
                    "⚾ The Pirates have won 5 World Series championships: 1909, 1925, 1960, 1971, and 1979!"
                ];
                
                let mut rng = thread_rng();
                facts.choose(&mut rng).unwrap_or(&facts[0]).to_string()
            },
            _ => format!("🏴‍☠️ Ahoy matey! No matter who we're playing, the Pirates spirit lives on! ⚾")
        }
    }
}
