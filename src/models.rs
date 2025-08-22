use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub date: NaiveDate,
    pub time: String,
    pub location: String,
    pub home_team: String,
    pub snacks: Option<String>,
    pub livestream: Option<String>,
    pub scoreboard: Option<String>,
    pub pitch_count: Option<String>,
}

impl EventData {
    pub fn new(date: NaiveDate, time: String, location: String, home_team: String, snacks: String, livestream: String, scoreboard: String, pitch_count: String) -> Self {
        Self {
            date,
            time,
            location,
            home_team,
            snacks: if snacks.is_empty() { None } else { Some(snacks) },
            livestream: if livestream.is_empty() { None } else { Some(livestream) },
            scoreboard: if scoreboard.is_empty() { None } else { Some(scoreboard) },
            pitch_count: if pitch_count.is_empty() { None } else { Some(pitch_count) },
        }
    }
}

#[derive(Debug, Clone)]
pub struct CorrelatedEvent {
    pub event_date: NaiveDate,
    pub event_summary: String,
    pub data: EventData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupMeMessage {
    pub text: String,
    pub sender_type: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct GroupMePostMessage {
    pub bot_id: String,
    pub text: String,
}

// Google Sheets API models
#[derive(Debug, Deserialize)]
pub struct SheetsResponse {
    pub values: Option<Vec<Vec<String>>>,
}

#[derive(Debug)]
pub enum BotCommand {
    NextGame,                                    // @Bot next game
    NextGames(usize),                           // @Bot next 3 games
    NextGameCategory(String),                   // @Bot next game snacks
    LetsGo(String),                            // @Bot lets go pirates
    Volunteer(NaiveDate, String, String),      // @Bot volunteer snacks 2025-01-15 John
    ShowVolunteers(Option<NaiveDate>),          // @Bot volunteers [date]
    Commands,                                   // @Bot commands
}

impl EventData {
    /// Format a location as a Google Maps hyperlink
    pub fn format_location_with_link(&self) -> String {
        if self.location.is_empty() {
            "TBD".to_string()
        } else {
            // URL encode the location for Google Maps
            let encoded_location = urlencoding::encode(&self.location);
            format!("{} (https://maps.google.com/?q={})", self.location, encoded_location)
        }
    }
    
    pub fn get_field(&self, field_name: &str) -> Option<&String> {
        match field_name.to_lowercase().as_str() {
            "time" => Some(&self.time),
            "location" => Some(&self.location),
            "hometeam" | "home_team" | "home" => Some(&self.home_team),
            "snacks" => self.snacks.as_ref(),
            "livestream" => self.livestream.as_ref(),
            "scoreboard" => self.scoreboard.as_ref(),
            "pitchcount" | "pitch_count" => self.pitch_count.as_ref(),
            _ => None,
        }
    }
    
    /// Check if a volunteer role is available (not assigned)
    pub fn is_role_available(&self, role: &str) -> bool {
        match role.to_lowercase().as_str() {
            "snacks" => self.snacks.is_none(),
            "livestream" => self.livestream.is_none(), 
            "scoreboard" => self.scoreboard.is_none(),
            "pitchcount" | "pitch_count" => self.pitch_count.is_none(),
            _ => false,
        }
    }
    
    /// Assign a volunteer to a role
    pub fn assign_volunteer(&mut self, role: &str, person: &str) -> bool {
        match role.to_lowercase().as_str() {
            "snacks" if self.snacks.is_none() => {
                self.snacks = Some(person.to_string());
                true
            },
            "livestream" if self.livestream.is_none() => {
                self.livestream = Some(person.to_string());
                true
            },
            "scoreboard" if self.scoreboard.is_none() => {
                self.scoreboard = Some(person.to_string());
                true
            },
            "pitchcount" | "pitch_count" if self.pitch_count.is_none() => {
                self.pitch_count = Some(person.to_string());
                true
            },
            _ => false,
        }
    }
    
    pub fn format_all(&self) -> String {
        let mut details = String::new();
        
        details.push_str(&format!("Date: {}\n", self.date.format("%Y-%m-%d")));
        details.push_str(&format!("Time: {}\n", self.time));
        details.push_str(&format!("Location: {}\n", self.format_location_with_link()));
        details.push_str(&format!("Home Team: {}\n", self.home_team));
        
        details.push_str(&format!("Snacks: {}\n", 
            self.snacks.as_ref().unwrap_or(&"⚠️ NEEDED".to_string())));
        details.push_str(&format!("Livestream: {}\n", 
            self.livestream.as_ref().unwrap_or(&"⚠️ NEEDED".to_string())));
        details.push_str(&format!("Scoreboard: {}\n", 
            self.scoreboard.as_ref().unwrap_or(&"⚠️ NEEDED".to_string())));
        details.push_str(&format!("Pitch Count: {}\n", 
            self.pitch_count.as_ref().unwrap_or(&"⚠️ NEEDED".to_string())));
        
        details
    }
    
    /// Format available volunteer opportunities
    pub fn format_volunteer_needs(&self) -> String {
        let mut needs = Vec::new();
        
        if self.snacks.is_none() {
            needs.push("snacks");
        }
        if self.livestream.is_none() {
            needs.push("livestream");
        }
        if self.scoreboard.is_none() {
            needs.push("scoreboard");
        }
        if self.pitch_count.is_none() {
            needs.push("pitch_count");
        }
        
        if needs.is_empty() {
            "✅ All volunteer roles are filled!".to_string()
        } else {
            format!("⚠️ Still needed: {}", needs.join(", "))
        }
    }
}
