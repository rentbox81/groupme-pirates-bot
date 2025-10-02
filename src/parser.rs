use chrono::NaiveDate;
use crate::error::{BotError, Result};
use crate::models::BotCommand;
use crate::conversational_parser::{ConversationalParser, ParsedIntent};
use std::sync::{Arc, Mutex};

pub struct CommandParser {
    bot_name: String,
    failed_attempts: Arc<Mutex<u32>>,
    conversational_parser: ConversationalParser,
}

impl CommandParser {
    pub fn new(bot_name: String) -> Self {
        let conversational_parser = ConversationalParser::new(bot_name.clone());
        Self { 
            bot_name,
            failed_attempts: Arc::new(Mutex::new(0)),
            conversational_parser,
        }
    }

    pub fn parse_message(&self, text: &str) -> Result<Option<BotCommand>> {
        let text = text.trim();
        
        // Check if message is directed at the bot
        let bot_mention = format!("@{}", self.bot_name);
        if !text.to_lowercase().contains(&bot_mention.to_lowercase()) {
            return Ok(None);
        }

        // Try conversational parsing first
        if let Some(intent) = self.conversational_parser.parse_message(text) {
            return self.intent_to_command(intent, text);
        }

        // If no intent detected, shouldn't happen but return None
        Ok(None)
    }

    fn intent_to_command(&self, intent: ParsedIntent, original_text: &str) -> Result<Option<BotCommand>> {
        match intent {
            ParsedIntent::Volunteer { role, date, person, relative_game } => {
                self.handle_volunteer_intent(role, date, person, relative_game, original_text)
            }
            ParsedIntent::GameQuery { category, count, relative: _ } => {
                self.handle_game_query_intent(category, count)
            }
            ParsedIntent::VolunteerQuery { date } => {
                Ok(Some(BotCommand::ShowVolunteers(date)))
            }
            ParsedIntent::TeamSpirit => {
                Ok(Some(BotCommand::LetsGo("pirates".to_string())))
            }
            ParsedIntent::Help => {
                Ok(Some(BotCommand::Commands))
            }
            ParsedIntent::Unknown => {
                // Return a witty response instead of an error
                Err(BotError::InvalidCommand(self.conversational_parser.get_witty_response()))
            }
        }
    }

    fn handle_volunteer_intent(
        &self, 
        role: Option<String>, 
        date: Option<NaiveDate>, 
        person: Option<String>,
        relative_game: Option<usize>,
        _original_text: &str
    ) -> Result<Option<BotCommand>> {
        match (role, date, person, relative_game) {
            // Explicit date provided
            (Some(r), Some(d), Some(p), _) => {
                Ok(Some(BotCommand::Volunteer(d, r, p)))
            }
            // Relative game specified (e.g., "next game", "game after next")
            (Some(r), None, Some(p), Some(rel_game)) => {
                // Service layer will resolve the relative game index to actual date
                if rel_game == 0 {
                    Ok(Some(BotCommand::VolunteerNextGame(r, p)))
                } else {
                    // For "game after next", etc., we need a new command or fetch games here
                    // For now, use VolunteerNextGame and let service handle it
                    Ok(Some(BotCommand::VolunteerNextGame(r, p)))
                }
            }
            // No date or relative game - DEFAULT TO NEXT GAME
            (Some(r), None, Some(p), None) => {
                Ok(Some(BotCommand::VolunteerNextGame(r, p)))
            }
            // Missing role
            (None, _, Some(p), _) => {
                Err(BotError::InvalidCommand(
                    format!("🏴‍☠️ Thanks {}! What would you like to volunteer for? (snacks, livestream, scoreboard, or pitch count)", p)
                ))
            }
            // Missing person
            (Some(r), _, None, _) => {
                Err(BotError::InvalidCommand(
                    format!("🏴‍☠️ Great! Someone wants to do {}! Could you tell me your name?", r)
                ))
            }
            // Missing both role and person
            _ => {
                Err(BotError::InvalidCommand(
                    "🏴‍☠️ I think you want to volunteer! Tell me what role you'd like and your name, and I'll sign you up for the next game! 😊".to_string()
                ))
            }
        }
    }

    fn handle_game_query_intent(
        &self,
        category: Option<String>,
        count: Option<usize>
    ) -> Result<Option<BotCommand>> {
        match (category, count) {
            (Some(cat), _) => {
                // Specific category requested
                Ok(Some(BotCommand::NextGameCategory(cat)))
            }
            (None, Some(n)) => {
                // Multiple games requested
                if n > 0 && n <= 10 {
                    Ok(Some(BotCommand::NextGames(n)))
                } else {
                    Ok(Some(BotCommand::NextGames(3))) // Default to 3
                }
            }
            (None, None) => {
                // Just asking about the next game
                Ok(Some(BotCommand::NextGame))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_parser() -> CommandParser {
        CommandParser::new("TestBot".to_string())
    }

    #[test]
    fn test_conversational_volunteer() {
        let parser = create_parser();
        
        // These should be understood conversationally
        let result = parser.parse_message("@TestBot I've got snacks for Saturday John");
        assert!(result.is_ok());
    }

    #[test]
    fn test_conversational_game_query() {
        let parser = create_parser();
        
        let result = parser.parse_message("@TestBot when's the next game?");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Some(BotCommand::NextGame)));
    }

    #[test]
    fn test_unknown_intent_returns_friendly_message() {
        let parser = create_parser();
        
        let result = parser.parse_message("@TestBot blah blah random stuff");
        // Should return an error with a friendly message, not panic
        assert!(result.is_err());
        if let Err(BotError::InvalidCommand(msg)) = result {
            // Should contain a friendly response - check for common words
            assert!(msg.contains("🏴‍☠️") || msg.contains("⚾") || msg.contains("Ahoy") || msg.contains("help"));
        } else {
            panic!("Expected InvalidCommand error");
        }
    }

    #[test]
    fn test_help_intent() {
        let parser = create_parser();
        
        let result = parser.parse_message("@TestBot help");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Some(BotCommand::Commands)));
    }

    #[test]
    fn test_team_spirit() {
        let parser = create_parser();
        
        let result = parser.parse_message("@TestBot let's go pirates!");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Some(BotCommand::LetsGo(_))));
    }

    #[test]
    fn test_volunteer_next_game() {
        let parser = create_parser();
        
        let result = parser.parse_message("@TestBot Hobbs have snacks for the next game");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Some(BotCommand::VolunteerNextGame(_, _))));
    }
    
    #[test]
    fn test_volunteer_defaults_to_next_game() {
        let parser = create_parser();
        
        // No date specified - should default to next game
        let result = parser.parse_message("@TestBot Hobbs have snacks");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Some(BotCommand::VolunteerNextGame(_, _))));
    }
}
