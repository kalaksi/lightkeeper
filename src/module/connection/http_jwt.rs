use std::collections::HashMap;
use ureq;
use serde_derive::Deserialize;
use serde_json;

use std::sync::Mutex;
use lightkeeper_module::connection_module;
use crate::error::LkError;
use crate::module::*;
use crate::module::connection::*;
use crate::utils::string_validation::is_alphanumeric_with;

#[connection_module(
    name="http-jwt",
    version="0.0.1",
    cache_scope="Global",
    description="Sends a HTTP request and handles JWT authentication. Currently supports only anonymous authentication.",
)]
pub struct HttpJwt {
    agent: ureq::Agent,
    jwt_tokens: Mutex<HashMap<String, String>>,
}

impl Module for HttpJwt {
    fn new(_settings: &HashMap<String, String>) -> Self {
        HttpJwt {
            agent: ureq::Agent::new(),
            jwt_tokens: Mutex::new(HashMap::new()),
        }
    }
}

impl ConnectionModule for HttpJwt {
    fn send_message(&self, message: &str) -> Result<ResponseMessage, LkError> {
        if message.is_empty() {
            return Ok(ResponseMessage::empty());
        }

        let mut parts = message.split("\n");
        let url = parts.next().unwrap();
        let data = parts.next().unwrap_or_default();
        let domain = url.split("/").nth(2).unwrap_or_default();

        for _ in 0..2 {
            let token = self.jwt_tokens.lock().map_err(|_| LkError::other("Failed to lock JWT tokens."))?
                                       .get(domain).cloned();

            // Currently only supports GET and POST requests.
            let response = if data.is_empty() {
                let mut request = self.agent.get(url);
                if let Some(token) = token {
                    request = request.set("Authorization", &format!("Bearer {}", token));
                }
                request.call()
            } else {
                let mut request = self.agent.post(url);
                if let Some(token) = token {
                    request = request.set("Authorization", &format!("Bearer {}", token));
                }
                request.send_string(data)
            };


            if let Ok(response) = response {
                let response_string = response.into_string()?;
                return Ok(ResponseMessage::new_success(response_string));
            }
            else if let Err(ureq::Error::Status(status, response)) = response {
                if status != 401 {
                    return Err(LkError::other_p("HTTP request failed with status", &status));
                }

                let auth_url = Self::parse_challenge_header(&response)?;
                let auth_response = self.agent.get(&auth_url).call()
                                              .map_err(|_| LkError::other("JWT authentication failed."))?;

                let auth_response_string = auth_response.into_string()?;
                let jwt_response: JwtResponse = serde_json::from_str(&auth_response_string)
                    .map_err(|_| LkError::other("JWT authentication response is invalid."))?;

                log::debug!("JWT authentication successful for {}", domain);

                let mut jwt_tokens = self.jwt_tokens.lock().map_err(|_| LkError::other("Failed to lock JWT tokens."))?;

                // Make sure the token cache doesn't grow indefinitely.
                if jwt_tokens.len() > 200 {
                    jwt_tokens.clear();
                }

                jwt_tokens.insert(domain.to_string(), jwt_response.token);
            }
        }

        Err(LkError::other("JWT authentication failed."))
    }
}

impl HttpJwt {
    fn parse_challenge_header(response: &ureq::Response) -> Result<String, LkError> {
        let mut realm = None;
        let mut params = Vec::new();

        if let Some(challenge_header) = response.header("WWW-Authenticate") {
            let (bearer, challenge_parts) = challenge_header.split_once(" ").unwrap_or_default();

            if bearer.to_lowercase() != "bearer" {
                return Err(LkError::other("JWT authentication challenge header is missing."));
            }

            for part in challenge_parts.split(",") {
                if let Some((key, value)) = part.split_once("=") {
                    let key = key.trim();
                    let value = value.trim_matches('"');

                    if !is_alphanumeric_with(value, ".:/_-") {
                        return Err(LkError::other("JWT authentication challenge header contains invalid key or value."));
                    }

                    match key {
                        "realm" => realm = Some(value),
                        "service" => params.push(("service", value)),
                        "scope" => params.push(("scope", value)),
                        _ => log::warn!("Unknown JWT authentication challenge key: {}", key),
                    }
                }
            }
        }

        if realm.is_none() {
            return Err(LkError::other("JWT authentication challenge header is missing."));
        }

        let mut url = realm.unwrap().to_string();
    
        if !params.is_empty() {
            url.push_str("?");
            url.push_str(&params.iter().map(|(key, value)| format!("{}={}", key, value)).collect::<Vec<_>>().join("&"));
        }

        Ok(url)
    }
}


#[derive(Deserialize)]
struct JwtResponse {
    token: String,
}