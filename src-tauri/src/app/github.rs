use serde::{Deserialize, Serialize};
use tracing::info;

use crate::HTTP_CLIENT;

const GITHUB_OAUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_API_BASE: &str = "https://api.github.com";

#[derive(Serialize, Deserialize)]
pub struct GithubTokenResponse {
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "token_type")]
    pub token_type: String,
    pub scope: String,
}

#[derive(Serialize, Deserialize)]
pub struct GithubUser {
    pub login: String,
    pub id: u64,
    #[serde(rename = "avatar_url")]
    pub avatar_url: String,
}

/// Get the GitHub OAuth authorization URL with configurable client_id
pub fn get_auth_url(client_id: &str) -> String {
    format!(
        "{}?client_id={}&scope=repo&redirect_uri=http://127.0.0.1:42069/callback",
        GITHUB_OAUTH_URL, client_id
    )
}

/// Exchange an authorization code for an access token with configurable credentials
pub async fn exchange_code(code: &str, client_id: &str, client_secret: &str) -> Result<String, String> {
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
    ];

    let client = oauth2::reqwest::Client::new();
    let response = client
        .post(GITHUB_TOKEN_URL)
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to exchange code: {}", e))?;

    let token_response: GithubTokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    Ok(token_response.access_token)
}

/// Fetch the authenticated user's GitHub profile
pub async fn fetch_user(token: &str) -> Result<GithubUser, String> {
    let response = HTTP_CLIENT
        .get(format!("{}/user", GITHUB_API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "AMT-Client")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user: {}", e))?;

    let user: GithubUser = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse user: {}", e))?;

    Ok(user)
}

/// Trigger a repository dispatch event to update the backend
pub async fn trigger_dispatch(
    token: &str,
    repo_owner: &str,
    repo_name: &str,
    event_type: &str,
    payload: serde_json::Value,
) -> Result<(), String> {
    let body = serde_json::json!({
        "event_type": event_type,
        "client_payload": payload
    });

    let response = HTTP_CLIENT
        .post(format!(
            "{}/repos/{}/{}/dispatches",
            GITHUB_API_BASE, repo_owner, repo_name
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "AMT-Client")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to trigger dispatch: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Dispatch failed: {}", response.status()));
    }

    info!("Dispatched event: {} to {}/{}", event_type, repo_owner, repo_name);
    Ok(())
}
