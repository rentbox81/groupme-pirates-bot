use chrono::{NaiveDate, Utc, Datelike, Duration};
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Conversational parser that understands natural language
pub struct ConversationalParser {
    bot_name: String,
}

#[derive(Debug, Clone)]
pub enum ParsedIntent {
    Volunteer { roles: Vec<String>, date: Option<NaiveDate>, person: Option<String>, relative_game: Option<usize> },
    GameQuery { category: Option<String>, count: Option<usize>, relative: Option<String> },
    VolunteerQuery { date: Option<NaiveDate> },
    TeamSpirit,
    Help,
    Unknown,
    RemoveVolunteer { person: String, role: String, date: Option<NaiveDate> },
    AssignVolunteer { person: String, role: String, date: Option<NaiveDate> },
    AddModerator { user_id: String },
    RemoveModerator { user_id: String },
    ListModerators,
    ListBotMessages { count: usize },
    DeleteBotMessage { message_id: String },
    CleanBotMessages { count: usize },
    ConversationalResponse { message: String },
}

impl ConversationalParser {
    pub fn new(bot_name: String) -> Self {
        Self { bot_name }
    }

    /// Parse a message and extract intent
    pub fn parse_message(&self, text: &str, sender_name: Option<&str>, attachments: &[crate::models::Attachment]) -> Option<ParsedIntent> {
        let text = text.trim();
        let text_lower = text.to_lowercase();
        
        // Check if message is directed at the bot
        let bot_mention = format!("@{}", self.bot_name).to_lowercase();
        if !text_lower.contains(&bot_mention) {
            return None;
        }

        // Remove bot mention for easier parsing
        let cleaned_text = text_lower.replace(&bot_mention, "").trim().to_string();
        
        if cleaned_text.is_empty() {
            return Some(ParsedIntent::Help);
        }

        // Detect intent based on keywords and patterns
        let intent = self.detect_intent(&cleaned_text, text, sender_name, attachments);
        Some(intent)
    }

    fn detect_intent(&self, text_lower: &str, original_text: &str, sender_name: Option<&str>, attachments: &[crate::models::Attachment]) -> ParsedIntent {
        // Volunteer intent detection
        // Admin command detection (check first, before volunteer)
        if text_lower.contains("remove") && text_lower.contains("from") {
            return self.parse_remove_volunteer(text_lower);
        }
        if text_lower.contains("assign") && text_lower.contains("to") {
            return self.parse_assign_volunteer(text_lower);
        }
        if text_lower.contains("add moderator") || text_lower.contains("add mod") {
            return self.parse_add_moderator(text_lower, attachments);
        }
        if text_lower.contains("remove moderator") || text_lower.contains("remove mod") {
            return self.parse_remove_moderator(text_lower, attachments);
        }
        if text_lower.contains("list moderator") || text_lower.contains("show moderator") {
            return ParsedIntent::ListModerators;
        }

        // Message management commands
        if text_lower.contains("list") && (text_lower.contains("message") || text_lower.contains("bot message")) {
            return self.parse_list_messages(text_lower);
        }
        if text_lower.contains("delete") && text_lower.contains("message") {
            return self.parse_delete_message(text_lower);
        }
        if text_lower.contains("clean") && (text_lower.contains("message") || text_lower.contains("bot message")) {
            return self.parse_clean_messages(text_lower);
        }

        if self.is_volunteer_intent(text_lower) {
            return self.parse_volunteer_intent(text_lower, original_text, sender_name);
        }

        // Game query intent detection
        if self.is_game_query_intent(text_lower) {
            return self.parse_game_query_intent(text_lower);
        }

        // Volunteer query intent detection
        if self.is_volunteer_query_intent(text_lower) {
            return self.parse_volunteer_query_intent(text_lower);
        }

        // Team spirit intent detection
        if self.is_team_spirit_intent(text_lower) {
            return ParsedIntent::TeamSpirit;
        }

        // Help intent detection
        if self.is_help_intent(text_lower) {
            return ParsedIntent::Help;
        }
        // Conversational message detection
        if self.is_conversational_message(text_lower) {
            let message = self.get_conversational_response(text_lower);
            return ParsedIntent::ConversationalResponse { message };
        }


        // Unknown intent
        ParsedIntent::Unknown
    }

    // Volunteer intent detection
    fn is_volunteer_intent(&self, text: &str) -> bool {
        let volunteer_keywords = [
            "i've got", "i have", "i'll bring", "i can do", "i can bring",
            "put me down", "sign me up", "i'll do", "i'll take",
            "count me in", "i got", "i'm doing", "volunteer", "i can",
            "have got", "has got", "will bring", "will do"
        ];
        
        let role_keywords = ["snacks", "snack", "livestream", "stream", "scoreboard", "score", "pitchcount", "pitch count", "gamechanger", "game changer"];
        
        let has_volunteer_keyword = volunteer_keywords.iter().any(|kw| text.contains(kw));
        let has_role_keyword = role_keywords.iter().any(|kw| text.contains(kw));
        
        has_volunteer_keyword || has_role_keyword
    }

    fn parse_volunteer_intent(&self, text_lower: &str, original_text: &str, sender_name: Option<&str>) -> ParsedIntent {
        let roles = self.extract_volunteer_roless(text_lower);
        let date = self.extract_date(text_lower);
        let person = self.extract_person_name(original_text);
        let relative_game = self.extract_relative_game(text_lower);

        // If no person extracted from text, use sender's name as fallback
        let person = person.or_else(|| sender_name.map(|s| s.to_string()));

        ParsedIntent::Volunteer { roles, date, person, relative_game }
    }

    fn extract_volunteer_roless(&self, text: &str) -> Vec<String> {
        let role_mappings = [
            (vec!["snacks", "snack", "food", "treats"], "snacks"),
            (vec!["livestream", "stream", "streaming", "live"], "livestream"),
            (vec!["scoreboard", "score", "scoring", "gamechanger", "game changer"], "scoreboard"),
            (vec!["pitchcount", "pitch count", "pitch", "pitches"], "pitchcount"),
        ];

        let mut found_roles = Vec::new();

        for (keywords, role) in &role_mappings {

            if keywords.iter().any(|kw| text.contains(kw)) {
                found_roles.push(role.to_string());
            }
        }

        found_roles
    }

fn extract_person_name(&self, text: &str) -> Option<String> {
    // Words to exclude (pronouns, contractions, etc.)
    let excluded_words = [
        "i", "i've", "i'll", "i'm", "we", "we've", "we'll", "we're",
        "you", "you've", "you'll", "he", "she", "they", "it"
    ];
    
    let words: Vec<&str> = text.split_whitespace().collect();
    
    // Helper to check if word is excluded
    let is_excluded = |word: &str| -> bool {
        let word_lower = word.to_lowercase().trim_matches('\'').to_string();
        excluded_words.contains(&word_lower.as_str())
    };
    
    // Check for "for [Name]" pattern
    if let Some(for_idx) = words.iter().position(|&w| w.to_lowercase() == "for") {
        if for_idx + 1 < words.len() {
            let name_parts: Vec<&str> = words[for_idx + 1..].iter()
                .take_while(|w| w.chars().next().map_or(false, |c| c.is_uppercase()))
                .filter(|w| !is_excluded(w))
                .copied()
                .collect();
            if !name_parts.is_empty() {
                return Some(name_parts.join(" "));
            }
        }
    }

    // Check for "- [Name]" pattern
    if let Some(dash_idx) = words.iter().position(|&w| w == "-") {
        if dash_idx + 1 < words.len() {
            let name_parts: Vec<&str> = words[dash_idx + 1..].iter()
                .take_while(|w| w.chars().next().map_or(false, |c| c.is_uppercase()))
                .filter(|w| !is_excluded(w))
                .copied()
                .collect();
            if !name_parts.is_empty() {
                return Some(name_parts.join(" "));
            }
        }
    }

    // Look for capitalized words (potential names) - but exclude pronouns
    for (i, word) in words.iter().enumerate() {
        if word.chars().next().map_or(false, |c| c.is_uppercase()) 
            && !word.starts_with('@') 
            && word.len() > 1
            && !is_excluded(word) {
            // Collect consecutive capitalized words
            let name_parts: Vec<&str> = words[i..].iter()
                .take_while(|w| w.chars().next().map_or(false, |c| c.is_uppercase()))
                .filter(|w| !is_excluded(w))
                .copied()
                .collect();
            if !name_parts.is_empty() {
                return Some(name_parts.join(" "));
            }
        }
    }

    None
}

    fn extract_date(&self, text: &str) -> Option<NaiveDate> {
        let today = Utc::now().date_naive();
        
        // Relative date keywords
        if text.contains("today") {
            return Some(today);
        }
        if text.contains("tomorrow") {
            return Some(today + Duration::days(1));
        }

        // Day of week detection
        let weekdays = [
            ("monday", 0), ("mon", 0),
            ("tuesday", 1), ("tues", 1), ("tue", 1),
            ("wednesday", 2), ("wed", 2),
            ("thursday", 3), ("thurs", 3), ("thu", 3),
            ("friday", 4), ("fri", 4),
            ("saturday", 5), ("sat", 5),
            ("sunday", 6), ("sun", 6),
        ];

        for (day_name, target_weekday) in &weekdays {
            if text.contains(day_name) {
                let current_weekday = today.weekday().num_days_from_monday() as i64;
                let target = *target_weekday as i64;
                let mut days_ahead = target - current_weekday;
                
                // If the day has passed this week, go to next week
                if days_ahead <= 0 {
                    days_ahead += 7;
                }
                
                // If "next [day]" is mentioned, add another week
                if text.contains("next") && text.contains(day_name) {
                    days_ahead += 7;
                }
                
                return Some(today + Duration::days(days_ahead));
            }
        }

        // Try to parse explicit dates
        let date_formats = [
            "%Y-%m-%d", "%m/%d/%Y", "%m-%d-%Y", "%d/%m/%Y", "%Y/%m/%d",
            "%m/%d", "%m-%d" // Month/day without year
        ];

        let words: Vec<&str> = text.split_whitespace().collect();
        for word in words {
            for format in &date_formats {
                // Handle dates without year by adding current year
                if format.contains("%m/%d") && !format.contains("%Y") {
                    let with_year = format!("{}/{}", word, today.year());
                    if let Ok(date) = NaiveDate::parse_from_str(&with_year, "%m/%d/%Y") {
                        return Some(date);
                    }
                } else if let Ok(date) = NaiveDate::parse_from_str(word, format) {
                    return Some(date);
                }
            }
        }

        None
    }

    fn extract_relative_game(&self, text: &str) -> Option<usize> {
        // "next game" or just "next" = game 0 (next)
        if text.contains("next game") || (text.contains("next") && !text.contains("after")) {
            return Some(0);
        }
        
        // "game after next" = game 1
        if text.contains("game after next") || text.contains("after next") {
            return Some(1);
        }
        
        // "two games from now", "second game", etc.
        if text.contains("two games") || text.contains("2 games") || text.contains("second game") {
            return Some(1);
        }
        
        if text.contains("three games") || text.contains("3 games") || text.contains("third game") {
            return Some(2);
        }

        None
    }

    // Game query intent detection
    fn is_game_query_intent(&self, text: &str) -> bool {
        let query_keywords = [
            "next game", "next", "when", "what time", "where", "location",
            "schedule", "upcoming", "games"
        ];
        
        query_keywords.iter().any(|kw| text.contains(kw))
    }

    fn parse_game_query_intent(&self, text: &str) -> ParsedIntent {
        let category = self.extract_game_category(text);
        let count = self.extract_game_count(text);
        let relative = self.extract_relative_time(text);

        ParsedIntent::GameQuery { category, count, relative }
    }

    fn extract_game_category(&self, text: &str) -> Option<String> {
        let categories = [
            "time", "location", "where", "home", "snacks", 
            "livestream", "scoreboard", "pitchcount", "pitch count"
        ];

        for category in &categories {
            if text.contains(category) {
                return Some(category.to_string());
            }
        }

        None
    }

    fn extract_game_count(&self, text: &str) -> Option<usize> {
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            if let Ok(num) = word.parse::<usize>() {
                // Check if followed by "games"
                if i + 1 < words.len() && words[i + 1].contains("game") {
                    return Some(num);
                }
            }
        }

        // Check for word numbers
        let number_words = [
            ("one", 1), ("two", 2), ("three", 3), ("four", 4), ("five", 5),
            ("six", 6), ("seven", 7), ("eight", 8), ("nine", 9), ("ten", 10)
        ];

        for (word, num) in &number_words {
            if text.contains(word) && text.contains("game") {
                return Some(*num);
            }
        }

        None
    }

    fn extract_relative_time(&self, text: &str) -> Option<String> {
        if text.contains("next") {
            return Some("next".to_string());
        }
        if text.contains("upcoming") {
            return Some("upcoming".to_string());
        }
        None
    }

    // Volunteer query intent detection
    fn is_volunteer_query_intent(&self, text: &str) -> bool {
        let query_keywords = [
            "who", "who's", "volunteers", "volunteer status", "need", "needed",
            "available", "open", "assignments"
        ];
        
        let context_keywords = ["snacks", "livestream", "scoreboard", "pitchcount", "volunteer"];
        
        let has_query = query_keywords.iter().any(|kw| text.contains(kw));
        let has_context = context_keywords.iter().any(|kw| text.contains(kw));
        
        has_query && has_context
    }

    fn parse_volunteer_query_intent(&self, text: &str) -> ParsedIntent {
        let date = self.extract_date(text);
        ParsedIntent::VolunteerQuery { date }
    }

    // Team spirit intent detection
    fn is_team_spirit_intent(&self, text: &str) -> bool {
        let spirit_keywords = [
            "let's go", "lets go", "go pirates", "pirates", "spirit",
            "hype", "pump", "motivation", "fact"
        ];
        
        spirit_keywords.iter().any(|kw| text.contains(kw))
    }

    // Help intent detection
    fn is_help_intent(&self, text: &str) -> bool {
        let help_keywords = ["help", "commands", "what can you do", "how"];
        help_keywords.iter().any(|kw| text.contains(kw))
    }

    fn is_conversational_message(&self, text: &str) -> bool {
        let conversational_keywords = ["scared", "fear", "thank", "thanks", "hi", "hello", "funny", "lol"];
        conversational_keywords.iter().any(|kw| text.contains(kw))
    }

    fn get_conversational_response(&self, text: &str) -> String {
        if text.contains("scared") || text.contains("fear") { self.get_fear_response() }
        else if text.contains("thank") { self.get_thanks_response() }
        else if text.contains("funny") || text.contains("lol") { self.get_humor_response() }
        else { self.get_generic_conversational_response() }
    }

    fn get_fear_response(&self) -> String { "🏴‍☠️ No need to fear! I'm just here to help with baseball. ⚾".to_string() }
    fn get_humor_response(&self) -> String { "⚾ Humor setting: TARS level. 75%% honesty. 🤖".to_string() }
    fn get_thanks_response(&self) -> String { "🏴‍☠️ You're welcome! Happy to help. ⚾".to_string() }
    fn get_positive_response(&self) -> String { "🏴‍☠️ Thanks! I do my best. ⚾".to_string() }
    fn get_negative_response(&self) -> String { "🏴‍☠️ Sorry! Tell me how to improve. 🔧".to_string() }
    fn get_generic_conversational_response(&self) -> String { "🏴‍☠️ Hi! I help with schedules and volunteers. ⚾".to_string() }

    /// Generate a witty iPhone response
    pub fn get_witty_response(&self) -> String {
        let responses = [
            "🏴‍☠️ Ahoy! I'm not quite sure what you're asking, but I'm here to help! Try asking about the next game or volunteer to bring snacks! 🍪",
            "⚾ Hmm, that's a new one! Maybe ask me 'when's the next game?' or 'I've got snacks'? 🤔",
            "🏴‍☠️ I'm still learning pirate speak! Try asking me about games, volunteers, or say 'let's go Pirates!' 🏴‍☠️",
            "📱 iPhone autocorrect failing you? Shocking. Nobody could have predicted that except literally everyone. Try 'next game'! 💸",
            "⚾ Not quite sure what you mean, matey! Ask me about upcoming games or volunteer roles! 🏴‍☠️",
            "🏴‍☠️ Shiver me timbers! That's a puzzler. Try 'next game', 'I've got snacks', or 'let's go Pirates!' ⚾",
            "📱 Is that message from you or your iPhone's delusions of intelligence? Hard to tell. Try 'volunteers'! 🤡",
            "⚾ Arrr, I'm not sure what ye be sayin'! Ask about the next game or volunteer to help out! 🏴‍☠️",
            "📱 Sent from my iPhone (which explains everything). Try 'next game' - even iOS can handle that! 🙄",
            "📱 Your iPhone just randomly typed that? Must be the 'courage' Tim Cook talked about. Try 'show volunteers'! 🎪",
            "📱 'It just works'... at making typos! Thanks Steve! Now try 'when is the next game?' ⚾",
            "📱 iPhone 15 Pro Max and still can't type? That's $1200 of regret right there. Try 'next game'! 💀",
            "📱 Your iPhone's autocorrect is more confused than people who bought the $19 polishing cloth. Ask 'volunteers'? 🤦",
            "📱 Apple removed the headphone jack AND the ability to type coherently. Brave. Try 'next game' maybe? 🎭",
            "📱 'Think Different'? Your iPhone isn't thinking at all. Neither was your wallet apparently. Try 'volunteers'! 🤑",
            "📱 You paid Apple prices for Android reliability. Congrats! 🎉 Now try 'show volunteers' or 'next game'!",
            "📱 Your iPhone has the computing power of a 2010 laptop at 3x the price. And it STILL autocorrected that wrong. 'next game'? 🚀",
        ];

        let mut rng = thread_rng();
        responses.choose(&mut rng).unwrap_or(&responses[0]).to_string()
    }

    /// Generate a helpful suggestion based on partial understanding
    pub fn get_helpful_suggestion(&self, intent: &ParsedIntent) -> String {
        match intent {
            ParsedIntent::Volunteer { roles, date, person, relative_game } => {
                let mut suggestions = Vec::new();
                
                if roles.is_empty() {
                    suggestions.push("what you'd like to volunteer for (snacks, livestream, scoreboard, or pitch count)");
                }
                if date.is_none() && relative_game.is_none() {
                    suggestions.push("which game (like 'next game' or 'Saturday')");
                }
                if person.is_none() {
                    suggestions.push("your name");
                }
                
                if suggestions.is_empty() {
                    "🏴‍☠️ I think you want to volunteer! Let me check on that...".to_string()
                } else {
                    format!("🏴‍☠️ I think you want to volunteer! Could you also mention {}?", 
                           suggestions.join(" and "))
                }
            }
            ParsedIntent::GameQuery { category, .. } => {
                if category.is_none() {
                    "⚾ Looking for game info? Let me show you what's coming up!".to_string()
                } else {
                    format!("⚾ Looking for {} info? Here's what I found!", category.as_ref().unwrap())
                }
            }
            _ => "🏴‍☠️ Let me show you what I can help with!".to_string()
        }
    }

    fn parse_remove_volunteer(&self, text: &str) -> ParsedIntent {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut person = String::new();
        let mut role = String::new();
        
        if let Some(from_idx) = words.iter().position(|&w| w == "from") {
            if from_idx > 1 { person = words[1..from_idx].join(" "); }
            if from_idx + 1 < words.len() { role = words[from_idx + 1].to_string(); }
        }
        
        ParsedIntent::RemoveVolunteer { person, role, date: None }
    }

    fn parse_assign_volunteer(&self, text: &str) -> ParsedIntent {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut person = String::new();
        let mut role = String::new();
        
        if let Some(to_idx) = words.iter().position(|&w| w == "to") {
            if to_idx > 1 { person = words[1..to_idx].join(" "); }
            if to_idx + 1 < words.len() { role = words[to_idx + 1].to_string(); }
        }
        
        ParsedIntent::AssignVolunteer { person, role, date: None }
    }

    fn parse_add_moderator(&self, text: &str, attachments: &[crate::models::Attachment]) -> ParsedIntent {
        let user_id = attachments
            .iter()
            .find(|a| a.attachment_type == "mentions")
            .and_then(|a| a.user_ids.first())
            .map(|id| id.clone())
            .unwrap_or_else(|| text.split_whitespace().last().unwrap_or("").to_string());
        ParsedIntent::AddModerator { user_id }
    }
    fn parse_remove_moderator(&self, text: &str, attachments: &[crate::models::Attachment]) -> ParsedIntent {
        let user_id = attachments
            .iter()
            .find(|a| a.attachment_type == "mentions")
            .and_then(|a| a.user_ids.first())
            .map(|id| id.clone())
            .unwrap_or_else(|| text.split_whitespace().last().unwrap_or("").to_string());
        ParsedIntent::RemoveModerator { user_id }
    }

    fn parse_list_messages(&self, text: &str) -> ParsedIntent {
        // Extract count if specified (e.g., "list 10 messages")
        let count = text.split_whitespace()
            .find_map(|word| word.parse::<usize>().ok())
            .unwrap_or(10); // Default to 10 messages
        
        ParsedIntent::ListBotMessages { count }
    }

    fn parse_delete_message(&self, text: &str) -> ParsedIntent {
        // Extract message ID (usually a long number)
        let words: Vec<&str> = text.split_whitespace().collect();
        let message_id = words.iter()
            .find(|w| w.len() > 10 && w.chars().all(|c| c.is_numeric()))
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        ParsedIntent::DeleteBotMessage { message_id }
    }

    fn parse_clean_messages(&self, text: &str) -> ParsedIntent {
        // Extract count if specified (e.g., "clean 5 messages")
        let count = text.split_whitespace()
            .find_map(|word| word.parse::<usize>().ok())
            .unwrap_or(5); // Default to 5 messages
        
        ParsedIntent::CleanBotMessages { count }
    }
}

mod tests {
    use super::*;

    fn create_parser() -> ConversationalParser {
        ConversationalParser::new("PirateBot".to_string())
    }

    #[test]
    fn test_volunteer_intent_detection() {
        let parser = create_parser();
        
        let test_cases = vec![
            "@PirateBot I've got snacks",
            "@PirateBot I can do livestream for Saturday",
            "@PirateBot put me down for scoreboard",
            "@PirateBot I'll bring snacks for next game",
        ];

        for case in test_cases {
            let intent = parser.parse_message(case, None, &[]);
            assert!(matches!(intent, Some(ParsedIntent::Volunteer { .. })));
        }
    }

    #[test]
    fn test_game_query_detection() {
        let parser = create_parser();
        
        let test_cases = vec![
            "@PirateBot when's the next game?",
            "@PirateBot what time is the game?",
            "@PirateBot where are we playing?",
            "@PirateBot show me next 3 games",
        ];

        for case in test_cases {
            let intent = parser.parse_message(case, None, &[]);
            assert!(matches!(intent, Some(ParsedIntent::GameQuery { .. })));
        }
    }

    #[test]
    fn test_date_extraction() {
        let parser = create_parser();
        
        assert!(parser.extract_date("saturday").is_some());
        assert!(parser.extract_date("tomorrow").is_some());
        assert!(parser.extract_date("2025-01-15").is_some());
        assert!(parser.extract_date("friday").is_some());
    }

    #[test]
    fn test_role_extraction() {
        let parser = create_parser();
        
        assert_eq!(parser.extract_volunteer_roless("i've got snacks"), vec!["snacks".to_string()]);
        assert_eq!(parser.extract_volunteer_roless("livestream for me"), vec!["livestream".to_string()]);
        assert_eq!(parser.extract_volunteer_roless("scoreboard please"), vec!["scoreboard".to_string()]);
    }
    
    #[test]
    fn test_relative_game_extraction() {
        let parser = create_parser();
        
        assert_eq!(parser.extract_relative_game("next game"), Some(0));
        assert_eq!(parser.extract_relative_game("game after next"), Some(1));
        assert_eq!(parser.extract_relative_game("two games from now"), Some(1));
    }
}
