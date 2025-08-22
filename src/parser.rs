use chrono::NaiveDate;
use crate::error::{BotError, Result};
use crate::models::BotCommand;
use std::sync::{Arc, Mutex};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct CommandParser {
    bot_name: String,
    failed_attempts: Arc<Mutex<u32>>,
}

impl CommandParser {
    pub fn new(bot_name: String) -> Self {
        Self { 
            bot_name,
            failed_attempts: Arc::new(Mutex::new(0)),
        }
    }

    pub fn parse_message(&self, text: &str) -> Result<Option<BotCommand>> {
        let text = text.trim();
        let parts: Vec<&str> = text.split_whitespace().collect();
        
        if parts.is_empty() {
            return Ok(None);
        }

        // Check if message is directed at the bot
        let bot_mention = format!("@{}", self.bot_name);
        if !parts[0].eq_ignore_ascii_case(&bot_mention) {
            return Ok(None);
        }

        // If only bot mention, show commands
        if parts.len() == 1 {
            return Ok(Some(BotCommand::Commands));
        }

        let command = parts[1].to_lowercase();
        
        match command.as_str() {
            "commands" | "help" => Ok(Some(BotCommand::Commands)),
            
            "next" => {
                if parts.len() < 3 {
                    return Err(BotError::InvalidCommand(
                        "Use: @bot next game, @bot next 3 games, or @bot next game snacks".to_string()
                    ));
                }
                
                let second_arg = parts[2].to_lowercase();
                
                if second_arg == "game" {
                    // @bot next game [category]
                    if parts.len() > 3 {
                        let category = parts[3].to_lowercase();
                        if self.is_valid_category(&category) {
                            Ok(Some(BotCommand::NextGameCategory(category)))
                        } else {
                            Err(BotError::InvalidCommand(
                                format!("Invalid category '{}'. Valid: time, location, home, snacks, livestream, scoreboard, pitchcount", category)
                            ))
                        }
                    } else {
                        Ok(Some(BotCommand::NextGame))
                    }
                } else if let Ok(num) = second_arg.parse::<usize>() {
                    // @bot next 3 games
                    if parts.len() > 3 && parts[3].to_lowercase() == "games" {
                        if num > 0 && num <= 10 {
                            Ok(Some(BotCommand::NextGames(num)))
                        } else {
                            Err(BotError::InvalidCommand(
                                "Number of games must be between 1 and 10".to_string()
                            ))
                        }
                    } else {
                        Err(BotError::InvalidCommand(
                            "Use: @bot next 3 games (with 'games' at the end)".to_string()
                        ))
                    }
                } else {
                    Err(BotError::InvalidCommand(
                        "Use: @bot next game, @bot next 3 games, or @bot next game snacks".to_string()
                    ))
                }
            }
            
            "lets" => {
                // @bot lets go pirates
                if parts.len() >= 4 && parts[2].to_lowercase() == "go" {
                    let team = parts[3].to_lowercase();
                    Ok(Some(BotCommand::LetsGo(team)))
                } else {
                    Err(BotError::InvalidCommand(
                        "Use: @bot lets go pirates".to_string()
                    ))
                }
            }
            
            "volunteer" => {
                // Handle "@bot volunteer [role] [date] [person_name]"
                if parts.len() < 4 {
                    return Err(BotError::InvalidCommand(
                        "Volunteer command requires: @bot volunteer [role] [date] [person_name]".to_string()
                    ));
                }
                
                let role = parts[2].to_lowercase();
                let date_str = parts[3];
                
                // Validate role
                if !self.is_valid_volunteer_role(&role) {
                    return Err(BotError::InvalidCommand(
                        format!("Invalid volunteer role '{}'. Valid roles: snacks, livestream, scoreboard, pitchcount", role)
                    ));
                }
                
                // Parse date
                let date = self.parse_date(date_str)?;
                
                // Extract person name (rest of the message)
                let person_name = if parts.len() > 4 {
                    parts[4..].join(" ")
                } else {
                    return Err(BotError::InvalidCommand(
                        "Please provide your name: @bot volunteer [role] [date] [your_name]".to_string()
                    ));
                };
                
                Ok(Some(BotCommand::Volunteer(date, role, person_name)))
            }
            
            "volunteers" => {
                // Handle "@bot volunteers" or "@bot volunteers [date]"
                if parts.len() > 2 {
                    let date_str = parts[2];
                    let date = self.parse_date(date_str)?;
                    Ok(Some(BotCommand::ShowVolunteers(Some(date))))
                } else {
                    Ok(Some(BotCommand::ShowVolunteers(None)))
                }
            }
            
            _ => {
                // Handle unknown command with suggestion and possibly iPhone jokes
                self.handle_unknown_command(&command)
            }
        }
    }

    fn parse_date(&self, date_str: &str) -> Result<NaiveDate> {
        // Try multiple date formats
        let formats = [
            "%Y-%m-%d",     // 2023-12-25
            "%m/%d/%Y",     // 12/25/2023
            "%m-%d-%Y",     // 12-25-2023
            "%d/%m/%Y",     // 25/12/2023 (European format)
            "%Y/%m/%d",     // 2023/12/25
        ];

        for format in &formats {
            if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
                return Ok(date);
            }
        }

        Err(BotError::InvalidCommand("Invalid date format".to_string()))
    }
    
    fn is_valid_volunteer_role(&self, role: &str) -> bool {
        match role.to_lowercase().as_str() {
            "snacks" | "livestream" | "scoreboard" | "pitchcount" | "pitch_count" => true,
            _ => false,
        }
    }
    
    fn is_valid_category(&self, category: &str) -> bool {
        match category.to_lowercase().as_str() {
            "time" | "location" | "home" | "hometeam" | "home_team" | 
            "snacks" | "livestream" | "scoreboard" | "pitchcount" | "pitch_count" => true,
            _ => false,
        }
    }
    
    fn handle_unknown_command(&self, command: &str) -> Result<Option<BotCommand>> {
        // Increment failed attempts counter
        let mut attempts = self.failed_attempts.lock().unwrap();
        *attempts += 1;
        let current_attempts = *attempts;
        
        // List of valid commands for similarity matching
        let valid_commands = vec![
            "commands", "help", "next", "lets", "volunteer", "volunteers"
        ];
        
        // Find the most similar command
        let suggestion = self.find_most_similar_command(command, &valid_commands);
        
        let mut error_message = String::new();
        
        // Add iPhone joke if user has failed multiple times
        if current_attempts >= 3 && current_attempts % 2 == 1 {
            error_message.push_str(&self.get_random_iphone_joke());
            error_message.push_str("\n\n");
        }
        
        if let Some(suggested_cmd) = suggestion {
            error_message.push_str(&format!(
                "🏴‍☠️ Ahoy! Unknown command '{}'. Did you mean '@{} {}'?\n\n", 
                command, self.bot_name, suggested_cmd
            ));
        } else {
            error_message.push_str(&format!(
                "🏴‍☠️ Ahoy! Unknown command '{}'.\n\n", 
                command
            ));
        }
        
        // Always show available commands
        error_message.push_str(&format!(
            "⚾ Available {} Commands:\n\n\
             🏴‍☠️ Game Info:\n\
             • @{} next game - Full details for next game\n\
             • @{} next 3 games - Show next 3 games\n\
             • @{} next game snacks - Get snacks info for next game\n\n\
             🏴‍☠️ Team Spirit:\n\
             • @{} lets go pirates - Get a Pirates fact!\n\n\
             🏴‍☠️ Volunteers:\n\
             • @{} volunteer snacks 2025-01-15 John - Sign up to volunteer\n\
             • @{} volunteers - Show all volunteer needs\n\
             • @{} volunteers 2025-01-15 - Show needs for specific date\n\n\
             📋 Categories: time, location, home, snacks, livestream, scoreboard, pitchcount\n\
             🏴‍☠️ Raise the Jolly Roger! ⚾",
            self.bot_name,
            self.bot_name,
            self.bot_name,
            self.bot_name,
            self.bot_name,
            self.bot_name,
            self.bot_name,
            self.bot_name
        ));
        
        Err(BotError::InvalidCommand(error_message))
    }
    
    fn find_most_similar_command(&self, input: &str, commands: &[&str]) -> Option<String> {
        let input = input.to_lowercase();
        let mut best_match = None;
        let mut best_score = 0;
        
        for &cmd in commands {
            let score = self.calculate_similarity(&input, cmd);
            if score > best_score && score >= 2 { // Minimum similarity threshold
                best_score = score;
                best_match = Some(cmd.to_string());
            }
        }
        
        best_match
    }
    
    fn calculate_similarity(&self, a: &str, b: &str) -> usize {
        // Simple character-based similarity scoring
        let mut score = 0;
        
        // Check for exact substring matches
        if a.contains(b) || b.contains(a) {
            score += 5;
        }
        
        // Check for common starting characters
        let min_len = a.len().min(b.len());
        for i in 0..min_len {
            if a.chars().nth(i) == b.chars().nth(i) {
                score += 2;
            } else {
                break;
            }
        }
        
        // Check for common characters anywhere
        for char_a in a.chars() {
            if b.contains(char_a) {
                score += 1;
            }
        }
        
        score
    }
    
    fn get_random_iphone_joke(&self) -> String {
        let iphone_jokes = [
            "📱 Is your iPhone autocorrecting 'commands' to 'commends' again? Classic Apple! 🙄",
            "📱 Let me guess... your iPhone keyboard suggested something completely different? Shocking! 😏",
            "📱 Having trouble with that fancy iPhone keyboard? Maybe try typing slower! 🐌📱",
            "📱 iPhone user detected! Don't worry, we don't judge... much. 😉",
            "📱 Your iPhone's predictive text strikes again! At least it's consistently unhelpful! 📱💸",
            "📱 Is that an iPhone typo or did you actually mean to type that? Hard to tell these days! 🤷‍♂️",
            "📱 Fun fact: Android users have 47% fewer command typos. Just saying... 🤖",
            "📱 Your iPhone must have that premium 'expensive typo' feature enabled! 💰",
            "📱 Maybe if you hadn't spent $1200 on a phone, you could afford typing lessons! 📱💸",
            "📱 iPhone's SwiftKey keyboard: Swift to mess up your commands since 2007! ⚡️💥"
        ];
        
        let mut rng = thread_rng();
        iphone_jokes.choose(&mut rng).unwrap_or(&iphone_jokes[0]).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn create_parser() -> CommandParser {
        CommandParser::new("TestBot".to_string())
    }

    #[test]
    fn test_commands_help() {
        let parser = create_parser();
        
        // Test showing commands
        assert!(matches!(
            parser.parse_message("@TestBot").unwrap(),
            Some(BotCommand::Commands)
        ));
        
        assert!(matches!(
            parser.parse_message("@TestBot commands").unwrap(),
            Some(BotCommand::Commands)
        ));
        
        assert!(matches!(
            parser.parse_message("@TestBot help").unwrap(),
            Some(BotCommand::Commands)
        ));
    }

    #[test]
    fn test_next_game_commands() {
        let parser = create_parser();
        
        // Test "@bot next game"
        assert!(matches!(
            parser.parse_message("@TestBot next game").unwrap(),
            Some(BotCommand::NextGame)
        ));
        
        // Test "@bot next game snacks"
        if let Some(BotCommand::NextGameCategory(category)) = parser.parse_message("@TestBot next game snacks").unwrap() {
            assert_eq!(category, "snacks");
        } else {
            panic!("Expected NextGameCategory with snacks");
        }
    }

    #[test]
    fn test_next_games_commands() {
        let parser = create_parser();
        
        // Test "@bot next 3 games"
        if let Some(BotCommand::NextGames(num)) = parser.parse_message("@TestBot next 3 games").unwrap() {
            assert_eq!(num, 3);
        } else {
            panic!("Expected NextGames with 3");
        }
        
        // Test invalid number
        assert!(parser.parse_message("@TestBot next 0 games").is_err());
        assert!(parser.parse_message("@TestBot next 11 games").is_err());
    }

    #[test] 
    fn test_lets_go_commands() {
        let parser = create_parser();
        
        // Test "@bot lets go pirates"
        if let Some(BotCommand::LetsGo(team)) = parser.parse_message("@TestBot lets go pirates").unwrap() {
            assert_eq!(team, "pirates");
        } else {
            panic!("Expected LetsGo with pirates");
        }
        
        // Test invalid format
        assert!(parser.parse_message("@TestBot lets pirates").is_err());
    }

    #[test]
    fn test_ignore_non_mentions() {
        let parser = create_parser();
        
        assert!(parser.parse_message("hello world").unwrap().is_none());
        assert!(parser.parse_message("@OtherBot next").unwrap().is_none());
    }

    #[test]
    fn test_invalid_commands() {
        let parser = create_parser();
        
        assert!(parser.parse_message("@TestBot invalid").is_err());
    }
    
    #[test]
    fn test_volunteer_commands() {
        let parser = create_parser();
        let expected_date = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
        
        // Test valid volunteer command
        if let Some(BotCommand::Volunteer(date, role, person)) = 
            parser.parse_message("@TestBot volunteer snacks 2023-12-25 John Doe").unwrap() {
            assert_eq!(date, expected_date);
            assert_eq!(role, "snacks");
            assert_eq!(person, "John Doe");
        } else {
            panic!("Expected Volunteer command");
        }
        
        // Test invalid role
        assert!(parser.parse_message("@TestBot volunteer invalid_role 2023-12-25 John").is_err());
        
        // Test missing parameters
        assert!(parser.parse_message("@TestBot volunteer snacks").is_err());
        assert!(parser.parse_message("@TestBot volunteer snacks 2023-12-25").is_err());
    }
    
    #[test]
    fn test_volunteers_commands() {
        let parser = create_parser();
        let expected_date = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
        
        // Test show all volunteers
        assert!(matches!(
            parser.parse_message("@TestBot volunteers").unwrap(),
            Some(BotCommand::ShowVolunteers(None))
        ));
        
        // Test show volunteers for specific date
        if let Some(BotCommand::ShowVolunteers(Some(date))) = 
            parser.parse_message("@TestBot volunteers 2023-12-25").unwrap() {
            assert_eq!(date, expected_date);
        } else {
            panic!("Expected ShowVolunteers with date");
        }
    }
}
