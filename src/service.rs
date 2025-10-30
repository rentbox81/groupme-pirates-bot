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

#[derive(Clone)]
pub struct BotService {
    google_client: GoogleClient,
    groupme_client: GroupMeClient,
    config: Config,
    team_facts: Arc<TeamFactsProvider>,
    // Cache for event data to reduce API calls and enable volunteer modifications
    event_cache: Arc<RwLock<HashMap<NaiveDate, CorrelatedEvent>>>,
}

impl BotService {
    pub fn new(config: Config) -> Self {
        let google_client = GoogleClient::new(config.clone());
        let groupme_client = GroupMeClient::new(config.clone());
        
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
            config,
            team_facts,
            event_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn correlate_data(&self) -> Result<HashMap<NaiveDate, CorrelatedEvent>> {
        info!("Starting data correlation (sheets-first)");
        
        let sheets_data = self.google_client.get_sheets_data().await?;
        let calendar_events = self.google_client.get_calendar_events().await?;
        
        let mut correlated_map = HashMap::new();
        
        // Start with sheets data as the source of truth
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
        for (date, time, location, home_team, snacks, livestream, scoreboard, pitch_count) in sheets_data {
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
                // PRESERVE CALENDAR SUMMARY:                 
                // Update the event summary to be more descriptive using sheet data
                // PRESERVE CALENDAR SUMMARY:                 if !time.is_empty() && !home_team.is_empty() {
                // PRESERVE CALENDAR SUMMARY:                     event.event_summary = format!("{} - {}", time, home_team);
                // PRESERVE CALENDAR SUMMARY:                 }
                
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
                    format!("{} - {}", time, home_team)
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

    pub async fn handle_command(&self, command: BotCommand, sender_name: Option<&str>, user_id: Option<&str>, moderators_store: &crate::moderators::ModeratorsStore) -> Result<String> {
        match command {
            BotCommand::NextGame => {
                // @bot next game
                match self.find_next_event().await? {
                    Some(event) => {
                        let mut response = format!("{} Next Game: {}
", self.config.team_emoji, event.event_summary);
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
                
                let mut response = format!("{} Next {} Games:

", self.config.team_emoji, count.min(upcoming_events.len()));
                
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
            
            BotCommand::LetsGo(_team) => {
                // @bot lets go [team]
                Ok(self.team_facts.get_fact())
            }
            
            BotCommand::Volunteer(date, role, person) => {
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

\
                     {} Game Info:
\
                     • @{} next game - Full details for next game
\
                     • @{} next 3 games - Show next 3 games
\
                     • @{} next game snacks - Get snacks info for next game

\
                     {} Team Spirit:
\
                     • @{} lets go {} - {}

\
                     {} Volunteers:
\
                     • @{} volunteer snacks 2025-01-15 John - Sign up to volunteer
\
                     • @{} volunteers - Show all volunteer needs
\
                     • @{} volunteers 2025-01-15 - Show needs for specific date

\
                     📋 Categories: time, location, home, snacks, livestream, scoreboard, pitchcount
\
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
                Ok(format!("{} Removed {} from {} (date: {:?}) - Feature coming soon!", self.config.team_emoji, person, role, date))
            },
            BotCommand::AssignVolunteer(person, role, date) => {
                let user = user_id.ok_or(BotError::InvalidCommand("User ID required".to_string()))?;
                if !moderators_store.is_authorized(user, &self.config.admin_user_id).await {
                    return Err(BotError::InvalidCommand(format!("{} Only admins and moderators can assign volunteers", self.config.team_emoji)));
                }
                Ok(format!("{} Assigned {} to {} (date: {:?}) - Feature coming soon!", self.config.team_emoji, person, role, date))
            },
            BotCommand::AddModerator(new_mod_id) => {
                let user = user_id.ok_or(BotError::InvalidCommand("User ID required".to_string()))?;
                if !moderators_store.is_admin(user, &self.config.admin_user_id) {
                    return Err(BotError::InvalidCommand(format!("{} Only the admin can add moderators", self.config.team_emoji)));
                }
                moderators_store.add_moderator(new_mod_id.clone()).await;
                Ok(format!("{} Added moderator: {}", self.config.team_emoji, new_mod_id))
            },
            BotCommand::RemoveModerator(mod_id) => { let user = user_id.ok_or(BotError::InvalidCommand("User ID required".to_string()))?; if !moderators_store.is_admin(user, &self.config.admin_user_id) { return Err(BotError::InvalidCommand(format!("{} Only the admin can remove moderators", self.config.team_emoji))); } let removed = moderators_store.remove_moderator(&mod_id).await; if removed { Ok(format!("{} Removed moderator: {}", self.config.team_emoji, mod_id)) } else { Ok(format!("{} {} was not a moderator", self.config.team_emoji, mod_id)) } },
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
            },
        }
    }

    pub async fn send_response(&self, message: &str) -> Result<()> {
        self.groupme_client.send_message(message).await
    }
    
    async fn handle_volunteer_assignment(&self, date: NaiveDate, role: String, person: String, sender_name: Option<&str>) -> Result<String> {
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
                                Ok(message)
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
                                 current, role, date, event.format_matchup()))
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
                        let mut response = format!("{} Volunteer status for {} ({}):\n\n", 
                                                  self.config.team_emoji, date, event.format_matchup());
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
                    let mut response = format!("{} Volunteer status for upcoming events:\n\n", self.config.team_emoji);
                    
                    for (date, event) in upcoming_events.iter().take(5) {
                        response.push_str(&format!("{} ({}):\n", date, event.format_matchup()));
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

