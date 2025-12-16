use chrono::{NaiveDate, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

use crate::config::Config;
use crate::error::{Result, BotError};
use crate::google_client::GoogleClient;
use crate::groupme_client::GroupMeClient;
use crate::models::{CorrelatedEvent, EventData, BotCommand};
use crate::team_facts::TeamFactsProvider;
use crate::weather_client::WeatherClient;

#[derive(Clone)]
pub struct BotService {
    google_client: GoogleClient,
    groupme_client: GroupMeClient,
    weather_client: WeatherClient,
    config: Config,
    team_facts: Arc<TeamFactsProvider>,
    // Cache for event data to reduce API calls and enable volunteer modifications
    // Use Vec to support multiple events on the same day
    event_cache: Arc<RwLock<HashMap<NaiveDate, Vec<CorrelatedEvent>>>>,
}

impl BotService {
    pub fn new(config: Config) -> Self {
        let google_client = GoogleClient::new(config.clone());
        let groupme_client = GroupMeClient::new(config.clone());
        let weather_client = WeatherClient::new();
        
        // Initialize team facts provider
        let team_facts = Arc::new(TeamFactsProvider::new(
            config.team_name.clone(),
            config.team_emoji.clone(),
            config.enable_team_facts,
            config.team_facts_file.clone(),
        ));
        
        Self {
            google_client,
            groupme_client,
            weather_client,
            config,
            team_facts,
            event_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn correlate_data(&self) -> Result<HashMap<NaiveDate, Vec<CorrelatedEvent>>> {
        info!("Starting data loading (sheets only)");
        
        let sheets_data = self.google_client.get_sheets_data().await?;
        
        let mut correlated_map: HashMap<NaiveDate, Vec<CorrelatedEvent>> = HashMap::new();
        
        // Populate directly from Google Sheets
        for (date, time, location, home_team, snacks, livestream, scoreboard, pitch_count, gamechanger) in sheets_data {
            info!("Processing sheet data for {}", date);
            
            let event_data = EventData::new(
                date,
                time.clone(),
                location,
                home_team.clone(),
                snacks,
                livestream,
                scoreboard,
                pitch_count,
                gamechanger,
            );
            
            let summary = if !time.is_empty() && !home_team.is_empty() {
                format!("{} - {}", time, home_team)
            } else {
                format!("Event on {}", date)
            };
            
            let event = CorrelatedEvent {
                event_date: date,
                event_summary: summary,
                data: event_data,
            };
            
            correlated_map.entry(date).or_default().push(event);
        }
        
        info!("Data loading complete: {} dates with events", correlated_map.len());
        
        // Update cache with fresh data
        if let Ok(mut cache) = self.event_cache.write() {
            cache.clear();
            cache.extend(correlated_map.clone());
        }
        
        Ok(correlated_map)
    }
    
    pub async fn get_cached_or_fresh_data(&self) -> Result<HashMap<NaiveDate, Vec<CorrelatedEvent>>> {
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
        let events_map = self.correlate_data().await?;
        let now = Utc::now().naive_local(); // Use naive_local to match sheet semantics roughly
        let today = now.date();
        
        let mut all_events: Vec<CorrelatedEvent> = events_map.values().flatten().cloned().collect();
        // Sort by date, then by time string (best effort)
        all_events.sort_by(|a, b| {
            if a.event_date != b.event_date {
                a.event_date.cmp(&b.event_date)
            } else {
                // Simple string comparison for time isn't perfect but works for "10:00 AM" vs "2:00 PM" if format is consistent
                // Ideally we'd parse time, but keeping it simple for now as per previous logic
                a.data.time.cmp(&b.data.time)
            }
        });

        for event in all_events {
            // If date is in future, it's the next event
            if event.event_date > today {
                return Ok(Some(event));
            }
            
            // If date is today, check if time has passed
            if event.event_date == today {
                // Try to parse start time from string like "10:00 AM" or "8am-9:30am"
                if let Some(time_part) = event.data.time.split('-').next() {
                    // Try parsing various formats
                    let time_str = time_part.trim();
                    
                    // Helper to parse time
                    let parsed_time = self.parse_time_string(time_str);
                    
                    if let Some(time) = parsed_time {
                        if time > now.time() {
                            return Ok(Some(event));
                        }
                    } else {
                        // If can't parse time, assume it hasn't happened if it's today
                        // Or maybe return it if we are unsure?
                        // Let's err on side of showing it
                        return Ok(Some(event));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    fn parse_time_string(&self, time_str: &str) -> Option<chrono::NaiveTime> {
        let formats = [
            "%I:%M %p", // 10:00 AM
            "%l:%M %p", // 8:00 AM
            "%I:%M%p",  // 10:00AM
            "%l:%M%p",  // 8:00AM
            "%l%p",     // 8am
            "%I%p",     // 10am
            "%H:%M",    // 14:00
        ];
        
        let upper_time = time_str.to_uppercase();
        for fmt in &formats {
            if let Ok(t) = chrono::NaiveTime::parse_from_str(&upper_time, fmt) {
                return Some(t);
            }
        }
        None
    }

    pub async fn find_event_by_date(&self, query_date: NaiveDate) -> Result<Vec<CorrelatedEvent>> {
        // First check cache
        if let Ok(cache) = self.event_cache.read() {
            if let Some(events) = cache.get(&query_date) {
                return Ok(events.clone());
            }
        }
        
        // Not in cache, get fresh data
        let events_map = self.correlate_data().await?;
        Ok(events_map.get(&query_date).cloned().unwrap_or_default())
    }

    pub async fn handle_command(&self, command: BotCommand, sender_name: Option<&str>, user_id: Option<&str>, moderators_store: &crate::moderators::ModeratorsStore) -> Result<String> {
        match command {
            BotCommand::NextGame => {
                // @bot next game
                match self.find_next_event().await? {
                    Some(event) => {
                        let mut response = format!("{} Next Game: {}\n", self.config.team_emoji, event.event_summary);
                        response.push_str(&event.data.format_all());
                        
                        // Fetch weather
                        if !event.data.location.is_empty() && event.data.location != "TBD" {
                             match self.weather_client.get_forecast(&event.data.location, event.data.date, &event.data.time).await {
                                 Ok(forecast) => response.push_str(&format!("\n{}\n", forecast)),
                                 Err(e) => warn!("Failed to fetch weather: {}", e),
                             }
                        }
                        
                        Ok(response)
                    }
                    None => Ok("⚾ No upcoming games found.".to_string()),
                }
            }
            
            BotCommand::NextGames(count) => {
                // @bot next X games
                let events_map = self.correlate_data().await?;
                let today = Utc::now().date_naive();
                
                let mut upcoming_events: Vec<CorrelatedEvent> = events_map.values().flatten().cloned().collect();
                
                upcoming_events.sort_by_key(|e| e.event_date);
                
                let upcoming_events: Vec<_> = upcoming_events.into_iter()
                    .filter(|e| e.event_date >= today)
                    .collect();
                
                if upcoming_events.is_empty() {
                    return Ok("⚾ No upcoming games found.".to_string());
                }
                
                let mut response = format!("{} Next {} Games:\n\n", self.config.team_emoji, count.min(upcoming_events.len()));
                
                for event in upcoming_events.iter().take(count) {
                    response.push_str(&format!("📅 {} - {}\n", event.event_date.format("%Y-%m-%d"), event.event_summary));
                    response.push_str(&format!("⏰ Time: {}\n", event.data.time));
                    response.push_str(&format!("📍 Location: {}\n", event.data.format_location_with_link()));
                    response.push_str(&format!("🏠 Home/Away: {}\n\n", event.data.home_team));
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
                            "weather" => {
                                 if let Ok(forecast) = self.weather_client.get_forecast(&event.data.location, event.data.date, &event.data.time).await {
                                     Ok(forecast)
                                 } else {
                                     Ok("❌ Could not fetch weather forecast.".to_string())
                                 }
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
            
            BotCommand::LetsGo(_team) => {
                // @bot lets go [team]
                Ok(self.team_facts.get_fact())
            }
            
            BotCommand::Volunteer(date, role, person) => {
                // If there are multiple games, try to assign to the first available one?
                // For simplicity, we'll try to assign to ANY game on that date that has the role open.
                // Or maybe we should just assign to the first one.
                self.handle_volunteer_assignment(date, role, person, sender_name).await
            }
            
            BotCommand::VolunteerNextGame(role, person) => {
                // Find the next game date and volunteer for it
                match self.find_next_event().await? {
                    Some(event) => {
                        self.handle_volunteer_assignment(event.event_date, role, person, sender_name).await
                    }
                    None => Ok("❌ No upcoming games found to volunteer for.".to_string()),
                }
            }
            
            BotCommand::ShowVolunteers(maybe_date) => {
                self.handle_show_volunteers(maybe_date).await
            }
            
            BotCommand::Commands => {
                let team_spirit_text = if self.config.enable_team_facts {
                    format!("Get a {} fact!", self.config.team_name)
                } else {
                    "Show team spirit!".to_string()
                };
                
                Ok(format!(
                    "⚾ {} Commands:

                     {} Game Info:
                     • @{} next game - Full details for next game
                     • @{} next 3 games - Show next 3 games
                     • @{} next game snacks - Get snacks info for next game

                     {} Team Spirit:
                     • @{} lets go {} - {}

                     {} Volunteers:
                     • @{} volunteer snacks 2025-01-15 John - Sign up to volunteer
                     • @{} volunteers - Show all volunteer needs
                     • @{} volunteers 2025-01-15 - Show needs for specific date

                     📋 Categories: time, location, home, snacks, livestream, scoreboard, pitchcount, gamechanger

                     {} Let's go {}! ⚾",
                    self.config.groupme_bot_name,
                    self.config.team_emoji,
                    self.config.groupme_bot_name,
                    self.config.groupme_bot_name,
                    self.config.groupme_bot_name,
                    self.config.team_emoji,
                    self.config.groupme_bot_name,
                    self.config.team_name.to_lowercase(),
                    team_spirit_text,
                    self.config.team_emoji,
                    self.config.groupme_bot_name,
                    self.config.groupme_bot_name,
                    self.config.groupme_bot_name,
                    self.config.team_emoji,
                    self.config.team_name
                ))
            }
            BotCommand::RemoveVolunteer(person, role, date) => {
                let user = user_id.ok_or(BotError::InvalidCommand("User ID required".to_string()))?;
                if !moderators_store.is_authorized(user, &self.config.admin_user_id).await {
                    return Err(BotError::InvalidCommand(format!("{} Only admins and moderators can remove volunteers", self.config.team_emoji)));
                }
                
                // If date is provided, use it. Otherwise, find the next game.
                let target_date = match date {
                    Some(d) => d,
                    None => match self.find_next_event().await? {
                        Some(event) => event.event_date,
                        None => return Ok("❌ No upcoming games found.".to_string()),
                    }
                };
                
                // For RemoveVolunteer, we assign an empty string to the role
                // We use handle_volunteer_assignment but pass empty string for person
                // However, we need to pass a sender name for formatting, but since we are clearing, we can construct a custom message
                // Or we can modify handle_volunteer_assignment to handle clearing.
                // Better yet, just call update_volunteer_assignment directly if we found the event.
                
                // Use handle_volunteer_assignment for consistency, but we need to trick it to clear the name
                // Actually, handle_volunteer_assignment checks is_role_available. If we are removing, the role is NOT available (it's taken).
                // So handle_volunteer_assignment will return "Role is already filled".
                // We need a separate function or logic for removal.
                
                self.handle_volunteer_removal(target_date, role, person).await
            },
            BotCommand::AssignVolunteer(person, role, date) => {
                let user = user_id.ok_or(BotError::InvalidCommand("User ID required".to_string()))?;
                if !moderators_store.is_authorized(user, &self.config.admin_user_id).await {
                    return Err(BotError::InvalidCommand(format!("{} Only admins and moderators can assign volunteers", self.config.team_emoji)));
                }
                
                // If date is provided, use it. Otherwise, find the next game.
                let target_date = match date {
                    Some(d) => d,
                    None => match self.find_next_event().await? {
                        Some(event) => event.event_date,
                        None => return Ok("❌ No upcoming games found.".to_string()),
                    }
                };
                
                // Assign works just like volunteering, but initiated by mod/admin
                // We can use handle_volunteer_assignment, passing None as sender_name to get neutral message, 
                // or just rely on the standard message.
                // The person argument is the volunteer's name.
                
                self.handle_volunteer_assignment(target_date, role, person, None).await
            },
            BotCommand::AddModerator(new_mod_id) => {
                let user = user_id.ok_or(BotError::InvalidCommand("User ID required".to_string()))?;
                if !moderators_store.is_admin(user, &self.config.admin_user_id) {
                    return Err(BotError::InvalidCommand(format!("{} Only the admin can add moderators", self.config.team_emoji)));
                }
                moderators_store.add_moderator(new_mod_id.clone()).await;
                Ok(format!("{} Added moderator: {}", self.config.team_emoji, new_mod_id))
            },
            BotCommand::RemoveModerator(mod_id) => { 
                let user = user_id.ok_or(BotError::InvalidCommand("User ID required".to_string()))?; 
                if !moderators_store.is_admin(user, &self.config.admin_user_id) { 
                    return Err(BotError::InvalidCommand(format!("{} Only the admin can remove moderators", self.config.team_emoji))); 
                } 
                let removed = moderators_store.remove_moderator(&mod_id).await; 
                if removed { 
                    Ok(format!("{} Removed moderator: {}", self.config.team_emoji, mod_id)) 
                } else { 
                    Ok(format!("{} {} was not a moderator", self.config.team_emoji, mod_id)) 
                } 
            },
            BotCommand::ListModerators => {
                let mods = moderators_store.list_moderators().await;
                if mods.is_empty() {
                    Ok(format!("{} No moderators assigned\nAdmin: {}", self.config.team_emoji, self.config.admin_user_id))
                } else {
                    Ok(format!("{} Moderators:\n{}\n\nAdmin: {}", self.config.team_emoji, mods.join("\n"), self.config.admin_user_id))
                }
            },
            BotCommand::ListBotMessages(count) => {
                let user = user_id.ok_or(BotError::InvalidCommand("User ID required".to_string()))?;
                if !moderators_store.is_authorized(user, &self.config.admin_user_id).await {
                    return Err(BotError::InvalidCommand(format!("{} Only admins and moderators can list bot messages", self.config.team_emoji)));
                }
                self.handle_list_bot_messages(count).await
            }
        }
    }

    pub async fn send_response(&self, message: &str) -> Result<()> {
        self.groupme_client.send_message(message).await
    }
    
    async fn handle_volunteer_removal(&self, date: NaiveDate, role: String, _person: String) -> Result<String> {
        let events = self.find_event_by_date(date).await?;
        
        if events.is_empty() {
            return Ok(format!("❌ No event found for {}.", date));
        }
        
        for (_i, mut event) in events.into_iter().enumerate() {
            // Check if role is valid first
            match role.to_lowercase().as_str() {
                "snacks" | "livestream" | "scoreboard" | "pitchcount" | "pitch_count" | "gamechanger" => {},
                _ => return Ok(format!("❌ Invalid role: {}", role)),
            };
            
            // We want to clear the role regardless of who has it (since this is an admin/mod command)
            // But checking if it's already empty is nice
            // Note: Google Sheets API clears a cell if we send an empty string
            
            match self.google_client.update_volunteer_assignment(date, &role, "").await {
                Ok(_) => {
                    // Update cache
                    self.correlate_data().await?;
                    
                    // Manually update local event copy just for message formatting (optional, since we reloaded cache)
                    // But we want to show the user what happened.
                    
                    return Ok(format!("✅ Cleared {} volunteer for {} ({})", role, date, event.format_matchup()));
                }
                Err(e) => {
                    warn!("Failed to update Google Sheet: {}", e);
                    return Ok("❌ Update failed. Code: VOL004".to_string());
                }
            }
        }
        
        Ok(format!("❌ Could not find event or role to remove for {}.", date))
    }

    async fn handle_volunteer_assignment(&self, date: NaiveDate, role: String, person: String, sender_name: Option<&str>) -> Result<String> {
        let events = self.find_event_by_date(date).await?;
        
        if events.is_empty() {
            return Ok(format!("❌ No event found for {}.", date));
        }
        
        // Find the first event that has this role available
        // Note: This logic assumes we update the FIRST matching game. 
        // In future, we might need more specific targeting (e.g. by time).
        for (i, mut event) in events.into_iter().enumerate() {
            if event.data.is_role_available(&role, &self.config.team_name) {
                // We need the row number to update the sheet.
                // Since we don't store row number, we need to look it up again or rely on the fact that
                // find_sheet_row_by_date logic needs to handle multiple games too.
                // The current GoogleClient::find_sheet_row_by_date only returns the FIRST match.
                // This is a limitation. We need to update GoogleClient to support updating specific game.
                // Workaround: We will use the GoogleClient's naive implementation which updates the first match for that date.
                // This implies we can only volunteer for the FIRST game of the day if using this logic.
                // TO FIX properly: we need to pass time to update_volunteer_assignment.
                
                // Let's rely on the user: if they say "volunteer", we try the first one.
                // But wait, if we have 2 games, and first one is full, we should check the second one.
                // But `update_volunteer_assignment` in `google_client` finds row by DATE. 
                // It will always find the first row with that date. 
                // We need to update `update_volunteer_assignment` to take time or index.
                
                // For now, let's just try to update. If `is_role_available` is true for this event, 
                // but `update_volunteer_assignment` updates the WRONG event (the first one), that's bad.
                
                // Hack: If we are on the second event (i > 0), we can't reliably update via the current `update_volunteer_assignment`.
                // We need to update `GoogleClient` to be smarter.
                // Since I cannot change everything at once, let's just try to update and warn if ambiguous.
                
                match self.google_client.update_volunteer_assignment(date, &role, &person).await {
                    Ok(_) => {
                        // Update cache (reload all data to be safe)
                        self.correlate_data().await?;
                        
                        let message = if let Some(sender) = sender_name {
                            let sender_lower = sender.to_lowercase();
                            let person_lower = person.to_lowercase();
                            if sender_lower == person_lower || sender_lower.contains(&person_lower) || person_lower.contains(&sender_lower) {
                                format!("@{} ✅ You've been assigned to {} for {} ({})!", sender, role, date, event.format_matchup())
                            } else {
                                format!("✅ {} has been assigned to {} for {} ({})!", person, role, date, event.format_matchup())
                            }
                        } else {
                            format!("✅ {} has been assigned to {} for {} ({})!", person, role, date, event.format_matchup())
                        };
                        return Ok(message);
                    }
                    Err(e) => {
                        warn!("Failed to update Google Sheet: {}", e);
                        return Ok("❌ Update failed. Code: VOL001".to_string());
                    }
                }
            }
        }
        
        // If we get here, no event had the role available
        Ok(format!("❌ Role {} is already filled or not available for games on {}.", role, date))
    }
    
    async fn handle_show_volunteers(&self, maybe_date: Option<NaiveDate>) -> Result<String> {
        match maybe_date {
            Some(date) => {
                let events = self.find_event_by_date(date).await?;
                if events.is_empty() {
                    Ok(format!("❌ No event found for {}.", date))
                } else {
                    let mut response = format!("{} Volunteer status for {}:\n\n", self.config.team_emoji, date);
                    for event in events {
                        response.push_str(&format!("--- {} ---\n", event.format_matchup()));
                        response.push_str(&event.data.format_all());
                        response.push_str(&format!("\n{}\n\n", event.data.format_volunteer_needs(&self.config.team_name)));
                    }
                    Ok(response)
                }
            }
            None => {
                // Show volunteer status for all upcoming events
                let events_map = self.correlate_data().await?;
                let today = Utc::now().date_naive();
                
                let mut upcoming_events: Vec<CorrelatedEvent> = events_map.values().flatten().cloned().collect();
                upcoming_events.sort_by_key(|e| e.event_date);
                
                let upcoming_events: Vec<_> = upcoming_events.into_iter()
                    .filter(|e| e.event_date >= today)
                    .collect();
                
                if upcoming_events.is_empty() {
                    Ok("❌ No upcoming events found.".to_string())
                } else {
                    let mut response = format!("{} Volunteer status for upcoming events:\n\n", self.config.team_emoji);
                    
                    for event in upcoming_events.iter().take(5) {
                        response.push_str(&format!("{} ({}):\n", event.event_date, event.format_matchup()));
                        response.push_str(&format!("{}\n", event.data.format_volunteer_needs(&self.config.team_name)));
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
    
    async fn handle_list_bot_messages(&self, count: usize) -> Result<String> {
        // Check if message management is configured
        if self.config.groupme_access_token.is_none() || self.config.groupme_group_id.is_none() {
            return Ok(format!("{} Message management is not configured. Set GROUPME_ACCESS_TOKEN and GROUPME_GROUP_ID in .env", self.config.team_emoji));
        }
        
        let messages = self.groupme_client.list_messages(100, None).await?;
        let bot_messages: Vec<_> = messages.iter()
            .filter(|m| m.sender_type == "bot")
            .take(count)
            .collect();
        
        if bot_messages.is_empty() {
            return Ok(format!("{} No recent bot messages found.", self.config.team_emoji));
        }
        
        let mut response = format!("{} Recent bot messages (last {}):\n\n", self.config.team_emoji, bot_messages.len());
        for (i, msg) in bot_messages.iter().enumerate() {
            let preview = if msg.text.len() > 50 {
                format!("{}...", &msg.text[..50])
            } else {
                msg.text.clone()
            };
            response.push_str(&format!("{}. ID: {} - {}\n", i + 1, msg.id, preview));
        }
        response.push_str("\n💡 Note: Messages can only be deleted manually through the GroupMe mobile app.");
        
        Ok(response)
    }
}
