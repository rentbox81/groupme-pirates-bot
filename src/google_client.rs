use chrono::NaiveDate;
use reqwest::Client;
use tracing::{info, warn, error};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::auth::ServiceAccountAuth;

use crate::config::Config;
use crate::error::{BotError, Result};

#[derive(Clone)]
pub struct GoogleClient {
    client: Client,
    config: Config,
    service_auth: Option<Arc<Mutex<ServiceAccountAuth>>>,
}

impl GoogleClient {
    pub fn new(config: Config) -> Self {
        let service_auth = if let Ok(service_account_path) = std::env::var("GOOGLE_SERVICE_ACCOUNT_JSON") {
            match ServiceAccountAuth::new(&service_account_path) {
                Ok(auth) => {
                    tracing::info!("Service account authentication initialized successfully");
                    Some(Arc::new(Mutex::new(auth)))
                },
                Err(e) => {
                    tracing::warn!("Failed to initialize service account auth: {}", e);
                    None
                }
            }
        } else {
            tracing::info!("Using API key authentication (read-only)");
            None
        };

        Self {
            client: Client::new(),
            config,
            service_auth,
        }
    }

    pub async fn get_sheets_data(&self) -> Result<Vec<(NaiveDate, String, String, String, String, String, String, String, String)>> {
        let sheets_response: crate::models::SheetsResponse = if let Some(service_auth) = &self.service_auth {
            // Use service account authentication
            let mut auth = service_auth.lock().await;
            let access_token = auth.get_access_token().await?;
            
            let url = format!(
                "https://sheets.googleapis.com/v4/spreadsheets/{}/values/A2:I",
                &self.config.sheet_id
            );

            info!("Fetching sheet data from Google Sheets API (using service account)");

            let response = self.client
                .get(&url)
                .bearer_auth(access_token)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                error!("Sheets API request failed: {} - {}", status, error_text);
                return Err(BotError::GoogleApi(format!("Sheets API returned {}: {}", status, error_text)));
            }

            response.json().await?
        } else {
            // Fallback to API key method
            let url = format!(
                "https://sheets.googleapis.com/v4/spreadsheets/{}/values/A2:I?key={}",
                &self.config.sheet_id,
                &self.config.google_api_key
            );

            info!("Fetching sheet data from Google Sheets API (using API key)");

            let response = self.client
                .get(&url)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                error!("Sheets API request failed: {} - {}", status, error_text);
                return Err(BotError::GoogleApi(format!("Sheets API returned {}: {}", status, error_text)));
            }

            response.json().await?
        };

        info!("Sheet data retrieved: {} rows", 
            sheets_response.values.as_ref().map(|v| v.len()).unwrap_or(0));

        // Common parsing logic for both methods
        let values = sheets_response.values.unwrap_or_default();
        let mut parsed_data = Vec::new();
        
        for (row_idx, row) in values.iter().enumerate() {
            if row.len() >= 4 && !row[0].trim().is_empty() {
                match NaiveDate::parse_from_str(&row[0], "%Y-%m-%d") {
                    Ok(date) => {
                        let time = row.get(1).cloned().unwrap_or_default();
                        let location = row.get(2).cloned().unwrap_or_default();
                        let home_team = row.get(3).cloned().unwrap_or_default();
                        let snacks = row.get(4).cloned().unwrap_or_default();
                        let livestream = row.get(5).cloned().unwrap_or_default();
                        let scoreboard = row.get(6).cloned().unwrap_or_default();
                        let pitch_count = row.get(7).cloned().unwrap_or_default();
                        let gamechanger = row.get(8).cloned().unwrap_or_default();
                        
                        parsed_data.push((date, time, location, home_team, snacks, livestream, scoreboard, pitch_count, gamechanger));
                    }
                    Err(e) => {
                        warn!("Failed to parse date in row {}: {} - {}", row_idx + 2, row[0], e);
                    }
                }
            }
        }
        
        parsed_data.sort_by(|a, b| a.0.cmp(&b.0));
        
        info!("Parsed {} sheet rows", parsed_data.len());
        Ok(parsed_data)
    }

    /// Update a specific cell in the Google Sheet
    pub async fn update_sheet_cell(&self, row: usize, column: &str, value: &str) -> Result<()> {
        let range = format!("{}{}:{}{}", column, row, column, row);
        
        if let Some(service_auth) = &self.service_auth {
            // Use service account authentication
            let mut auth = service_auth.lock().await;
            let access_token = auth.get_access_token().await?;
            
            let url = format!(
                "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}?valueInputOption=RAW",
                &self.config.sheet_id,
                urlencoding::encode(&range)
            );

            let update_data = serde_json::json!({
                "values": [[value]]
            });

            info!("Updating sheet cell {}{} with value: {} (using service account)", column, row, value);
            
            let response = self.client
                .put(&url)
                .bearer_auth(access_token)
                .json(&update_data)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                error!("Sheet update failed: {} - {}", status, error_text);
                return Err(BotError::GoogleApi(format!("Sheet update returned {}: {}", status, error_text)));
            }

            info!("Successfully updated sheet cell {}{}", column, row);
            Ok(())
        } else {
            // Fallback to API key (read-only) with clear error message
            warn!("Write operation attempted with API key - requires service account");
            Err(BotError::GoogleApi("Write operations require service account authentication".to_string()))
        }
    }
    /// Find the row number for a specific date in the sheet
    pub async fn find_sheet_row_by_date(&self, target_date: chrono::NaiveDate) -> Result<Option<usize>> {
        let sheets_data = self.get_sheets_data().await?;
        
        for (index, (date, _title, _location, _home_team, _snacks, _livestream, _scoreboard, _pitch_count, _gamechanger)) in sheets_data.iter().enumerate() {
            if *date == target_date {
                // Row numbers are 1-indexed, and we start from row 2 (header is row 1)
                return Ok(Some(index + 2));
            }
        }
        
        Ok(None)
    }
    
    /// Update volunteer assignment in the sheet
    pub async fn update_volunteer_assignment(&self, date: chrono::NaiveDate, role: &str, person: &str) -> Result<()> {
        let row = self.find_sheet_row_by_date(date).await?
            .ok_or_else(|| BotError::InvalidCommand(format!("No event found for {}", date)))?;
            
        let column = match role.to_lowercase().as_str() {
            "snacks" => "E",
            "livestream" => "F", 
            "scoreboard" => "G",
            "pitchcount" | "pitch_count" => "H",
            "gamechanger" => "I",
            _ => return Err(BotError::InvalidCommand(format!("Invalid volunteer role: {}", role))),
        };
        
        self.update_sheet_cell(row, column, person).await
    }
}
