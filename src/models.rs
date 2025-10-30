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

#[derive(Debug, Deserialize, Clone)]
pub struct Attachment {
    #[serde(rename = "type")]
    pub attachment_type: String,
    #[serde(default)]
    pub user_ids: Vec<String>,
    #[serde(default)]
    pub loci: Vec<Vec<i32>>,
}

#[derive(Debug, Deserialize)]
pub struct GroupMeMessage {
    pub text: String,
    pub sender_type: String,
    pub name: String,
    pub user_id: String,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GroupMeMessageInfo {
    pub id: String,
    pub text: String,
    pub name: String,
    pub user_id: String,
    pub sender_type: String,
    pub created_at: i64,
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
    VolunteerNextGame(String, String), // role, person - resolved to next game
    RemoveVolunteer(String, String, Option<NaiveDate>), // person, role, date
    AssignVolunteer(String, String, Option<NaiveDate>), // person, role, date
    AddModerator(String), // user_id
    RemoveModerator(String), // user_id
    ListModerators,
    ListBotMessages(usize), // count - list recent bot messages
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

impl CorrelatedEvent {
    /// Parse and format the matchup from the calendar summary
    /// Returns a formatted string like "Pirates vs Dragons" or falls back to home team
    pub fn format_matchup(&self) -> String {
        if let Some((team1, team2)) = Self::parse_matchup(&self.event_summary) {
            // Determine which team is home based on home_team field
            let home_team_lower = self.data.home_team.to_lowercase();
            
            if team1.to_lowercase().contains(&home_team_lower) || home_team_lower.contains(&team1.to_lowercase()) {
                format!("{} (Home) vs {}", team1, team2)
            } else if team2.to_lowercase().contains(&home_team_lower) || home_team_lower.contains(&team2.to_lowercase()) {
                format!("{} vs {} (Home)", team1, team2)
            } else {
                // Can't determine home team, just show matchup
                format!("{} vs {}", team1, team2)
            }
        } else {
            // Fallback: construct a friendly description from available data
            if !self.data.home_team.is_empty() && self.data.home_team.to_lowercase() != "home" {
                format!("{} Game", self.data.home_team)
            } else if !self.data.time.is_empty() && !self.data.location.is_empty() {
                format!("{} at {}", self.data.time, self.data.location)
            } else if !self.event_summary.is_empty() {
                self.event_summary.clone()
            } else {
                "Game".to_string()
            }
        }
    }
    
    /// Parse matchup from calendar summary
    /// The calendar format from TeamSideline is: " Vs [OpponentTeam] - [Field] ([HomeTeam] - [Coach])"
    /// Example: " Vs Chaos 8U - Hall (Pirates - Hines)"
    /// Returns (HomeTeam, OpponentTeam) tuple
    fn parse_matchup(summary: &str) -> Option<(String, String)> {
        let summary = summary.trim();
        let summary_lower = summary.to_lowercase();
        
        // Look for the TeamSideline pattern: "Vs [Team] - [Field] ([HomeTeam] - [Coach])"
        if summary_lower.starts_with("vs ") {
            // Find the first " - " (separates opponent from field)
            if let Some(first_dash_pos) = summary.find(" - ") {
                // Extract opponent team (between "Vs " and first " - ")
                let opponent = summary[3..first_dash_pos].trim().to_string();
                
                // Find parentheses that contain home team info
                if let Some(paren_start) = summary.find('(') {
                    if let Some(paren_end) = summary.find(')') {
                        // Extract content inside parentheses
                        let paren_content = &summary[paren_start + 1..paren_end];
                        
                        // Find dash inside parentheses (separates home team from coach)
                        if let Some(dash_in_paren) = paren_content.find(" - ") {
                            let home_team = paren_content[..dash_in_paren].trim().to_string();
                            
                            if !opponent.is_empty() && !home_team.is_empty() {
                                return Some((home_team, opponent));
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback: try the old format for backward compatibility
        // "Team1 vs Team2" or similar patterns
        if let Some(vs_pos) = summary_lower.find(" vs ") {
            let before_vs = summary[..vs_pos].trim();
            let after_vs = summary[vs_pos + 4..].trim();
            
            let team1 = Self::extract_team_name(before_vs);
            let team2 = Self::extract_team_name(after_vs);
            
            if !team1.is_empty() && !team2.is_empty() {
                return Some((team1, team2));
            }
        }
        
        None
    }
    
    fn extract_team_name(text: &str) -> String {
        let text = text.trim();
        
        // If there's a dash, take everything after the last dash
        if let Some(dash_pos) = text.rfind('-') {
            let after_dash = text[dash_pos + 1..].trim();
            if !after_dash.is_empty() {
                return Self::clean_team_name(after_dash);
            }
        }
        
        Self::clean_team_name(text)
    }
    
    fn clean_team_name(text: &str) -> String {
        let text = text.trim();
        // Remove parenthetical info like "(Home)"
        if let Some(paren_pos) = text.find('(') {
            text[..paren_pos].trim().to_string()
        } else {
            text.to_string()
        }
    }
}
