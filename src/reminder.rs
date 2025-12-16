use chrono::{Local, Timelike};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration as TokioDuration};
use tracing::{info, warn, error};

use crate::config::Config;
use crate::service::BotService;

/// Tracks which reminders have been sent to avoid duplicates
#[derive(Default)]
pub struct ReminderState {
    sent_24h_reminders: HashSet<String>,  // game_date as string
    sent_15m_reminders: HashSet<String>,
}

pub struct ReminderScheduler {
    bot_service: Arc<BotService>,
    state: Arc<RwLock<ReminderState>>,
    config: Config,
    team_facts: Option<Arc<crate::team_facts::TeamFactsProvider>>,
}

impl ReminderScheduler {
    pub fn new(config: Config) -> Self {
        let bot_service = Arc::new(BotService::new(config.clone()));
        let state = Arc::new(RwLock::new(ReminderState::default()));
        
        // Initialize team facts for 15m reminder
        let team_facts = if config.enable_team_facts {
            Some(Arc::new(crate::team_facts::TeamFactsProvider::new(
                config.team_name.clone(),
                config.team_emoji.clone(),
                config.enable_team_facts,
                config.team_facts_file.clone(),
            )))
        } else {
            None
        };
        
        Self {
            bot_service,
            state,
            config,
            team_facts,
        }
    }

    /// Start the reminder scheduler in the background
    pub fn start(self: Arc<Self>) {
        let start_hour = self.config.reminder_start_hour;
        let end_hour = self.config.reminder_end_hour;
        
        tokio::spawn(async move {
            info!("Reminder scheduler started (active hours: {}:00 - {}:00)", start_hour, end_hour);
            
            loop {
                // Check every 5 minutes
                sleep(TokioDuration::from_secs(300)).await;
                
                if let Err(e) = self.check_and_send_reminders().await {
                    error!("Error checking reminders: {}", e);
                }
            }
        });
    }

    /// Check if current time is within acceptable reminder hours
    fn is_within_reminder_hours(&self) -> bool {
        let now = Local::now().naive_local();
        let current_hour = now.hour();
        
        // Check if current hour is within the configured range
        current_hour >= self.config.reminder_start_hour && current_hour < self.config.reminder_end_hour
    }

    async fn check_and_send_reminders(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if we're within acceptable reminder hours
        if !self.is_within_reminder_hours() {
            // Silently skip - don't send reminders too early or too late
            return Ok(());
        }

        let now = Local::now().naive_local();
        
        // ALWAYS fetch fresh data for reminders
        // Use find_next_event logic manually or adapt to new structure
        // Since find_next_event now handles the logic of finding the next relevant game, we can use it.
        // But reminder scheduler logic was previously iterating.
        // Let's rewrite to use correlate_data() which returns HashMap<Date, Vec<Event>>
        
        match self.bot_service.correlate_data().await {
            Ok(events_map) => {
                // Flatten and sort all events
                let mut all_events: Vec<crate::models::CorrelatedEvent> = events_map.values().flatten().cloned().collect();
                all_events.sort_by(|a, b| a.event_date.cmp(&b.event_date));
                
                let today = chrono::Utc::now().date_naive();
                
                // Find next event (same logic as service.rs basically)
                let mut next_event: Option<crate::models::CorrelatedEvent> = None;
                
                for event in all_events {
                    if event.event_date >= today {
                        // Check if time has passed if it is today
                        if event.event_date == today {
                             // Try parse time
                             if let Ok(dt) = self.parse_game_datetime(&event.event_date, &event.data.time) {
                                 if dt > now {
                                     next_event = Some(event);
                                     break; 
                                 }
                             } else {
                                 // If can't parse, assume future
                                 next_event = Some(event);
                                 break;
                             }
                        } else {
                            // Future date
                            next_event = Some(event);
                            break;
                        }
                    }
                }
                
                if let Some(event) = next_event {
                    // Use a unique key including time if possible, or just date/time string
                    let game_key = format!("{}T{}", event.event_date, event.data.time);
                    
                    // Skip reminder if time is TBD/unknown
                    if event.data.time.trim().is_empty() || event.data.time.trim().eq_ignore_ascii_case("TBD") {
                        info!("Skipping reminder for {} - time is TBD", game_key);
                        return Ok(());
                    }
                    
                    // Parse game time to get exact datetime
                    let game_datetime = match self.parse_game_datetime(&event.event_date, &event.data.time) {
                        Ok(dt) => dt,
                        Err(e) => {
                            warn!("Could not parse time '{}' for game {}: {}", event.data.time, game_key, e);
                            return Ok(());
                        }
                    };
                    
                    let time_until_game = game_datetime.signed_duration_since(now);
                    info!("Game datetime parsed: {} (date: {}, time: {}), Current time: {}, Minutes until game: {}", 
                        game_datetime, event.event_date, event.data.time, now, time_until_game.num_minutes());
                
                    // Check for 24-hour reminder
                    if time_until_game.num_hours() <= 24 && time_until_game.num_hours() > 23 {
                        let should_send = {
                            let state = self.state.read().await;
                            !state.sent_24h_reminders.contains(&game_key)
                        };
                        
                        if should_send {
                            info!("Sending 24-hour reminder for game on {} (current hour: {})", game_key, now.hour());
                            self.send_24h_reminder(&event).await?;
                            let mut state = self.state.write().await;
                            state.sent_24h_reminders.insert(game_key.clone());
                        }
                    }
                    
                    // Check for 15-minute reminder
                    if time_until_game.num_minutes() <= 15 && time_until_game.num_minutes() > 0 {
                        let should_send = {
                            let state = self.state.read().await;
                            !state.sent_15m_reminders.contains(&game_key)
                        };
                        
                        if should_send {
                            info!("Sending 15-minute reminder for game on {} (current hour: {})", game_key, now.hour());
                            self.send_15m_reminder(&event).await?;
                            let mut state = self.state.write().await;
                            state.sent_15m_reminders.insert(game_key);
                        }
                    }
                    
                    // Cleanup old reminders (games that have passed)
                    self.cleanup_old_reminders().await;
                } else {
                    info!("No upcoming games found for reminders");
                }
            }
            Err(e) => {
                warn!("Error fetching game data for reminders: {}", e);
            }
        }
        
        Ok(())
    }

    async fn send_24h_reminder(&self, event: &crate::models::CorrelatedEvent) -> Result<(), Box<dyn std::error::Error>> {
        let matchup = event.format_matchup();
        let mut message = format!("⏰ Game Reminder! 24 hours until:\n\n{} {}\n", self.config.team_emoji, matchup);
        message.push_str(&event.data.format_all());
        message.push_str("\n");
        message.push_str(&event.data.format_volunteer_needs(&self.config.team_name));
        
        self.bot_service.send_response(&message).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    async fn send_15m_reminder(&self, _event: &crate::models::CorrelatedEvent) -> Result<(), Box<dyn std::error::Error>> {
        let mut message = format!("⚾ Game starting in 15 minutes! {}\n\n", self.config.team_emoji);
        
        // Add a team fact if enabled
        if let Some(ref facts) = self.team_facts {
            message.push_str(&facts.get_fact());
            message.push_str("\n\n");
        }
        
        message.push_str(&format!("⚾ Let's go {}! {}", self.config.team_name, self.config.team_emoji));
        
        self.bot_service.send_response(&message).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    fn parse_game_datetime(&self, date: &chrono::NaiveDate, time_str: &str) -> Result<chrono::NaiveDateTime, Box<dyn std::error::Error>> {
        // Try to parse time from string (e.g., "10:00 AM", "14:30", etc.)
        let time_formats = [
            "%I:%M %p",  // 10:00 AM
            "%I:%M%p",   // 10:00AM
            "%H:%M",     // 14:30
            "%H:%M:%S",  // 14:30:00
        ];
        
        for format in &time_formats {
            if let Ok(time) = chrono::NaiveTime::parse_from_str(time_str.trim(), format) {
                return Ok(date.and_time(time));
            }
        }
        
        // If parsing fails, return error instead of defaulting
        Err(format!("Could not parse time: '{}'", time_str).into())
    }

    async fn cleanup_old_reminders(&self) {
        let now = Local::now().naive_local().date();
        
        let mut state = self.state.write().await;
        // Remove reminders for games that are more than 1 day old
        state.sent_24h_reminders.retain(|game_date| {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(game_date, "%Y-%m-%d") {
                (date - now).num_days() >= -1
            } else {
                false
            }
        });
        
        state.sent_15m_reminders.retain(|game_date| {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(game_date, "%Y-%m-%d") {
                (date - now).num_days() >= -1
            } else {
                false
            }
        });
    }
}
