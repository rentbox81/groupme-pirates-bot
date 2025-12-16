use reqwest::Client;
use serde::Deserialize;
use chrono::NaiveDate;
use crate::error::{BotError, Result};
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[derive(Debug, Deserialize)]
struct GeocodingResult {
    latitude: f64,
    longitude: f64,
    name: String,
    admin1: Option<String>, // State/Region
}

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    hourly: HourlyWeather,
    hourly_units: HourlyUnits,
}

#[derive(Debug, Deserialize)]
struct HourlyWeather {
    time: Vec<String>,
    temperature_2m: Vec<f64>,
    precipitation_probability: Vec<f64>,
    weather_code: Vec<i32>,
}

#[derive(Debug, Deserialize)]
struct HourlyUnits {
    temperature_2m: String,
}

#[derive(Clone)]
pub struct WeatherClient {
    client: Client,
}

impl WeatherClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_forecast(&self, location: &str, date: NaiveDate, time_str: &str) -> Result<String> {
        // 1. Geocode location
        let (lat, lon, location_name) = self.geocode(location).await?;
        
        // 2. Parse game time to find relevant forecast hour
        // time_str expected format: "HH:MM AM/PM" or "HH:MM"
        // We need to construct a target datetime to match against hourly forecast
        let hour_offset = self.parse_hour_from_time(time_str).unwrap_or(12); // Default to noon if parse fails
        
        // 3. Fetch weather
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m,precipitation_probability,weather_code&temperature_unit=fahrenheit&start_date={}&end_date={}&timezone=auto",
            lat, lon, date, date
        );
        
        info!("Fetching weather for {} ({}, {}) on {}", location_name, lat, lon, date);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(BotError::GoogleApi(format!("Weather API failed: {}", response.status())));
        }
        
        let weather_data: WeatherResponse = response.json().await?;
        
        // Find index for the game time (approximate)
        // API returns hourly data starting from 00:00 local time
        // So index = hour (0-23)
        let index = hour_offset as usize;
        
        if index < weather_data.hourly.time.len() {
            let temp = weather_data.hourly.temperature_2m[index];
            let precip = weather_data.hourly.precipitation_probability[index];
            let code = weather_data.hourly.weather_code[index];
            let unit = &weather_data.hourly_units.temperature_2m;
            
            let condition = self.weather_code_to_string(code);
            
            Ok(format!("🌡️ Forecast for {}: {:.1}{} - {}, 💧 {}% precip", 
                location_name, temp, unit, condition, precip))
        } else {
            Ok("Weather data not available for this time.".to_string())
        }
    }
    
    async fn geocode(&self, location: &str) -> Result<(f64, f64, String)> {
        // Strategy 1: Try content inside parentheses (often City)
        // e.g. "Field 1 (Plano)" -> "Plano"
        if let (Some(start), Some(end)) = (location.find('('), location.find(')')) {
            if start < end {
                let inner = location[start+1..end].trim();
                if !inner.is_empty() {
                    if let Ok(result) = self.fetch_geocoding(inner).await {
                        return Ok(result);
                    }
                }
            }
        }

        // Strategy 2: Try comma-based heuristics (City, State)
        // e.g. "123 Main St, McKinney, TX" -> "McKinney"
        if let Some((left, _)) = location.rsplit_once(',') {
            let parts: Vec<&str> = left.split_whitespace().collect();
            // Try last 1, 2, and 3 words (to handle "New York", "San Francisco")
            for i in 1..=3 {
                if parts.len() >= i {
                    let potential_city = parts[parts.len()-i..].join(" ");
                    if let Ok(result) = self.fetch_geocoding(&potential_city).await {
                        return Ok(result);
                    }
                }
            }
        }

        // Strategy 3: Try cleaned location (stripping parens)
        let clean_location = if let Some(idx) = location.find('(') {
            location[..idx].trim()
        } else {
            location.trim()
        };

        if !clean_location.is_empty() && clean_location.to_uppercase() != "TBD" {
             if let Ok(result) = self.fetch_geocoding(clean_location).await {
                 return Ok(result);
             }
        }
        
        // Fallback or error
        let msg = format!("Location not found: {}", location);
        warn!("{}", msg);
        Err(BotError::GoogleApi(msg))
    }

    async fn fetch_geocoding(&self, query: &str) -> Result<(f64, f64, String)> {
        let url = format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
            urlencoding::encode(query)
        );
        
        let response = self.client.get(&url).send().await?;
        let geo_data: GeocodingResponse = response.json().await?;
        
        if let Some(results) = geo_data.results {
            if let Some(first) = results.first() {
                let name = if let Some(admin) = &first.admin1 {
                    format!("{}, {}", first.name, admin)
                } else {
                    first.name.clone()
                };
                return Ok((first.latitude, first.longitude, name));
            }
        }
        Err(BotError::GoogleApi("No results".to_string()))
    }
    
    fn parse_hour_from_time(&self, time_str: &str) -> Option<u32> {
        // Try parsing "3:30 PM", "10:00 AM", "14:00"
        // Simple heuristic parsing
        let lower = time_str.to_lowercase();
        let is_pm = lower.contains("pm") || lower.contains("p.m.");
        
        // Extract numbers
        let parts: Vec<&str> = lower.split(|c: char| !c.is_numeric()).filter(|s| !s.is_empty()).collect();
        
        if let Some(hour_str) = parts.first() {
            if let Ok(mut hour) = hour_str.parse::<u32>() {
                if is_pm && hour < 12 {
                    hour += 12;
                } else if !is_pm && hour == 12 {
                    hour = 0; // 12 AM
                }
                return Some(hour);
            }
        }
        
        None
    }
    
    fn weather_code_to_string(&self, code: i32) -> String {
        match code {
            0 => "Clear sky",
            1..=3 => "Partly cloudy",
            45 | 48 => "Foggy",
            51..=55 => "Drizzle",
            56 | 57 => "Freezing Drizzle",
            61..=65 => "Rain",
            66 | 67 => "Freezing Rain",
            71..=75 => "Snow",
            77 => "Snow grains",
            80..=82 => "Rain showers",
            85 | 86 => "Snow showers",
            95 => "Thunderstorm",
            96 | 99 => "Thunderstorm with hail",
            _ => "Unknown",
        }.to_string()
    }
}
