use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};
use reqwest::Client;
use crate::error::{BotError, Result};

#[derive(Debug, Deserialize)]
pub struct ServiceAccountKey {
    pub client_email: String,
    pub private_key: String,
    pub token_uri: String,
}

#[derive(Debug, Serialize)]
struct Claims {
    iss: String,
    scope: String,
    aud: String,
    exp: u64,
    iat: u64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

pub struct ServiceAccountAuth {
    key: ServiceAccountKey,
    client: Client,
    cached_token: Option<(String, u64)>, // (token, expires_at)
}

impl ServiceAccountAuth {
    pub fn new(key_path: &str) -> Result<Self> {
        let key_content = std::fs::read_to_string(key_path)
            .map_err(|e| BotError::GoogleApi(format!("Failed to read service account key: {}", e)))?;
        
        let key: ServiceAccountKey = serde_json::from_str(&key_content)
            .map_err(|e| BotError::GoogleApi(format!("Failed to parse service account key: {}", e)))?;

        Ok(Self {
            key,
            client: Client::new(),
            cached_token: None,
        })
    }

    pub async fn get_access_token(&mut self) -> Result<String> {
        // Check if we have a valid cached token
        if let Some((token, expires_at)) = &self.cached_token {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            if *expires_at > now + 60 { // 60 second buffer
                return Ok(token.clone());
            }
        }

        // Generate new token
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let claims = Claims {
            iss: self.key.client_email.clone(),
            scope: "https://www.googleapis.com/auth/spreadsheets".to_string(),
            aud: self.key.token_uri.clone(),
            iat: now,
            exp: now + 3600, // 1 hour
        };

        let header = Header::new(Algorithm::RS256);
        
        // Clean the private key
        let private_key = self.key.private_key
            .replace("\\n", "\n");
        
        let encoding_key = EncodingKey::from_rsa_pem(private_key.as_bytes())
            .map_err(|e| BotError::GoogleApi(format!("Failed to create encoding key: {}", e)))?;

        let jwt = encode(&header, &claims, &encoding_key)
            .map_err(|e| BotError::GoogleApi(format!("Failed to encode JWT: {}", e)))?;

        // Exchange JWT for access token
        let mut params = HashMap::new();
        params.insert("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer");
        params.insert("assertion", &jwt);

        let response = self.client
            .post(&self.key.token_uri)
            .form(&params)
            .send()
            .await
            .map_err(|e| BotError::GoogleApi(format!("Token request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(BotError::GoogleApi(format!("Token request failed: {}", error_text)));
        }

        let token_response: TokenResponse = response.json().await
            .map_err(|e| BotError::GoogleApi(format!("Failed to parse token response: {}", e)))?;

        let expires_at = now + token_response.expires_in;
        self.cached_token = Some((token_response.access_token.clone(), expires_at));

        Ok(token_response.access_token)
    }
}
