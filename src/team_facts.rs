use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTeamFacts {
    pub team_name: String,
    pub facts: Vec<String>,
}

pub struct TeamFactsProvider {
    team_name: String,
    team_emoji: String,
    custom_facts: Option<CustomTeamFacts>,
    enabled: bool,
}

impl TeamFactsProvider {
    pub fn new(team_name: String, team_emoji: String, enabled: bool, facts_file: Option<String>) -> Self {
        let custom_facts = if let Some(path) = facts_file {
            Self::load_custom_facts(&path)
        } else {
            None
        };

        Self {
            team_name,
            team_emoji,
            custom_facts,
            enabled,
        }
    }

    fn load_custom_facts(path: &str) -> Option<CustomTeamFacts> {
        if Path::new(path).exists() {
            if let Ok(contents) = fs::read_to_string(path) {
                if let Ok(facts) = serde_json::from_str::<CustomTeamFacts>(&contents) {
                    return Some(facts);
                }
            }
        }
        None
    }

    pub fn get_fact(&self) -> String {
        if !self.enabled {
            return format!("{} Let's go team! ⚾", self.team_emoji);
        }

        // Try custom facts first
        if let Some(ref custom) = self.custom_facts {
            let mut rng = thread_rng();
            if let Some(fact) = custom.facts.choose(&mut rng) {
                return fact.clone();
            }
        }

        // Fall back to built-in facts
        self.get_builtin_fact()
    }

    fn get_builtin_fact(&self) -> String {
        let team_lower = self.team_name.to_lowercase();
        let mut rng = thread_rng();

        match team_lower.as_str() {
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
                facts.choose(&mut rng).unwrap_or(&facts[0]).to_string()
            },
            "yankees" => {
                let facts = [
                    "🗽 The New York Yankees have won 27 World Series championships, more than any other MLB team!",
                    "⚾ Babe Ruth hit 714 home runs in his career, with 659 of them as a Yankee!",
                    "🗽 The Yankees' pinstripe uniforms have been iconic since 1912!",
                    "⚾ Yankees Stadium is known as 'The House That Ruth Built' and opened in 1923!",
                    "🗽 Derek Jeter played his entire 20-year career with the Yankees and got 3,465 hits!",
                    "⚾ The Yankees retired more numbers than any other team - 22 different players and managers!",
                    "🗽 Joe DiMaggio's 56-game hitting streak in 1941 is still an MLB record!",
                    "⚾ The Yankees have had 44 players inducted into the Baseball Hall of Fame!"
                ];
                facts.choose(&mut rng).unwrap_or(&facts[0]).to_string()
            },
            "red sox" | "redsox" => {
                let facts = [
                    "🧦 The Boston Red Sox won their first World Series in 1903!",
                    "⚾ Fenway Park opened in 1912 and is the oldest ballpark in Major League Baseball!",
                    "🧦 The Green Monster at Fenway is 37 feet tall and one of baseball's most iconic features!",
                    "⚾ Ted Williams was the last player to bat over .400 in a season, hitting .406 in 1941!",
                    "🧦 The Red Sox broke the 'Curse of the Bambino' by winning the 2004 World Series!",
                    "⚾ David Ortiz, 'Big Papi', hit 541 career home runs, all with the Red Sox!",
                    "🧦 The Red Sox have won 9 World Series championships!",
                    "⚾ Carl Yastrzemski won the Triple Crown in 1967, leading in batting average, home runs, and RBIs!"
                ];
                facts.choose(&mut rng).unwrap_or(&facts[0]).to_string()
            },
            "cubs" => {
                let facts = [
                    "🐻 The Chicago Cubs broke a 108-year championship drought by winning the 2016 World Series!",
                    "⚾ Wrigley Field opened in 1914 and is the second-oldest ballpark in MLB!",
                    "🐻 The Cubs' ivy-covered outfield walls at Wrigley are iconic!",
                    "⚾ Ernie Banks, 'Mr. Cub', hit 512 home runs all with the Cubs!",
                    "🐻 The Cubs were founded in 1876, making them one of the oldest teams in baseball!",
                    "⚾ The Cubs have won 3 World Series championships: 1907, 1908, and 2016!",
                    "🐻 Sammy Sosa hit 609 home runs in his career, with 545 as a Cub!",
                    "⚾ The famous 'Curse of the Billy Goat' was believed to hex the Cubs for 71 years!"
                ];
                facts.choose(&mut rng).unwrap_or(&facts[0]).to_string()
            },
            "dodgers" => {
                let facts = [
                    "⚾ The Los Angeles Dodgers have won 7 World Series championships!",
                    "💙 Jackie Robinson broke baseball's color barrier with the Dodgers in 1947!",
                    "⚾ Dodger Stadium opened in 1962 and is the third-oldest ballpark in MLB!",
                    "💙 Sandy Koufax pitched 4 no-hitters including a perfect game in 1965!",
                    "⚾ The Dodgers moved from Brooklyn to Los Angeles in 1958!",
                    "💙 Clayton Kershaw has won 3 Cy Young Awards with the Dodgers!",
                    "⚾ Vin Scully announced Dodgers games for 67 years from 1950-2016!",
                    "💙 The Dodgers have had 55,000+ attendance records at Dodger Stadium!"
                ];
                facts.choose(&mut rng).unwrap_or(&facts[0]).to_string()
            },
            "giants" => {
                let facts = [
                    "🧡 The San Francisco Giants have won 8 World Series championships!",
                    "⚾ Willie Mays, 'The Say Hey Kid', hit 660 home runs and is considered one of the greatest players ever!",
                    "🧡 The Giants moved from New York to San Francisco in 1958!",
                    "⚾ Barry Bonds hit a record 762 career home runs!",
                    "🧡 Oracle Park (formerly AT&T Park) opened in 2000 with views of San Francisco Bay!",
                    "⚾ The Giants won 3 World Series in 5 years (2010, 2012, 2014)!",
                    "🧡 The Giants were founded in 1883 as the New York Gothams!",
                    "⚾ Juan Marichal was inducted into the Hall of Fame in 1983 after winning 238 games!"
                ];
                facts.choose(&mut rng).unwrap_or(&facts[0]).to_string()
            },
            "braves" => {
                let facts = [
                    "🪓 The Atlanta Braves franchise is the oldest continuously operating professional sports franchise in America!",
                    "⚾ Hank Aaron hit 755 home runs, most of them with the Braves!",
                    "🪓 The Braves won 14 consecutive division titles from 1991-2005!",
                    "⚾ The Braves have won 4 World Series championships!",
                    "🪓 Greg Maddux won 4 consecutive Cy Young Awards (1992-1995) with the Braves!",
                    "⚾ The Braves franchise has played in Boston, Milwaukee, and Atlanta!",
                    "🪓 Chipper Jones played his entire 19-year career with the Braves!",
                    "⚾ The 'Tomahawk Chop' has been a Braves tradition since 1991!"
                ];
                facts.choose(&mut rng).unwrap_or(&facts[0]).to_string()
            },
            _ => {
                // Generic response for teams without built-in facts
                format!("{} Go {}! Let's bring the energy and win this game! ⚾", 
                       self.team_emoji, self.team_name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pirates_facts() {
        let provider = TeamFactsProvider::new("Pirates".to_string(), "🏴‍☠️".to_string(), true, None);
        let fact = provider.get_fact();
        assert!(!fact.is_empty());
        assert!(fact.contains("Pirates") || fact.contains("⚾") || fact.contains("🏴‍☠️"));
    }

    #[test]
    fn test_disabled_facts() {
        let provider = TeamFactsProvider::new("Pirates".to_string(), "🏴‍☠️".to_string(), false, None);
        let fact = provider.get_fact();
        assert_eq!(fact, "🏴‍☠️ Let's go team! ⚾");
    }

    #[test]
    fn test_generic_team() {
        let provider = TeamFactsProvider::new("Dragons".to_string(), "🐉".to_string(), true, None);
        let fact = provider.get_fact();
        assert!(fact.contains("Dragons"));
        assert!(fact.contains("🐉"));
    }
}
