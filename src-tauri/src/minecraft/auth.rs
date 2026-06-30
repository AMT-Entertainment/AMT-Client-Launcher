use anyhow::Result;

use azalea_auth::{
    cache::ExpiringValue, get_minecraft_token, get_profile, AccessTokenResponse, AuthError,
    MinecraftAuthResponse, ProfileResponse, XboxLiveAuth,
};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tracing::{error, trace};

use uuid::Uuid;

use crate::HTTP_CLIENT;

/// The client ID of the Azure app used for authentication
pub(crate) const AZURE_CLIENT_ID: &str = "0add8caf-2cc6-4546-b798-c3d171217dd9";
const AZURE_SCOPE: &str = "XboxLive.signin offline_access";

const DEVICE_CODE_URL: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MinecraftAccount {
    #[serde(rename = "Premium")]
    MsaAccount {
        msa: ExpiringValue<AccessTokenResponse>,
        xbl: ExpiringValue<XboxLiveAuth>,
        mca: ExpiringValue<MinecraftAuthResponse>,
        #[serde(flatten)]
        profile: ProfileResponse,
    },
    #[serde(rename = "Microsoft")]
    LegacyMsaAccount {
        name: String,
        uuid: Uuid,
        token: String,
        ms_auth: MsAuth,
    },
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsAuth {
    pub expires_in: i64,
    pub access_token: String,
    pub refresh_token: String,
    #[serde(skip)]
    pub expires_after: i64,
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[allow(dead_code)]
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_description: Option<String>,
}

impl MinecraftAccount {
    pub async fn auth_msa<F>(on_code: F) -> Result<MinecraftAccount>
    where
        F: Fn(&String, &String),
    {
        let (verification_uri, user_code, device_code, interval) = get_device_code()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        on_code(&verification_uri, &user_code);

        let token = poll_for_token(&device_code, interval)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let expires_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + token.expires_in.unwrap_or(0);

        let msa = ExpiringValue {
            data: AccessTokenResponse {
                access_token: token.access_token.unwrap_or_default(),
                refresh_token: token.refresh_token.unwrap_or_default(),
                expires_in: token.expires_in.unwrap_or(0),
                scope: AZURE_SCOPE.to_string(),
                token_type: "Bearer".to_string(),
                user_id: String::new(),
            },
            expires_at,
        };

        Ok(login_msa(msa).await?)
    }

    pub async fn refresh(self) -> Result<MinecraftAccount> {
        match self {
            MinecraftAccount::MsaAccount {
                msa,
                xbl,
                mca,
                profile,
                ..
            } => {
                if !mca.is_expired() {
                    return Ok(MinecraftAccount::MsaAccount {
                        msa,
                        xbl,
                        mca,
                        profile,
                    });
                }

                let msa = if msa.is_expired() {
                    trace!("refreshing Microsoft auth token");
                    match refresh_msa_token(&msa.data.refresh_token).await {
                        Ok(new_msa) => new_msa,
                        Err(e) => {
                            error!("Error refreshing Microsoft auth token: {}", e);
                            msa
                        }
                    }
                } else {
                    msa
                };

                Ok(login_msa(msa).await?)
            }
            MinecraftAccount::LegacyMsaAccount { ms_auth, .. } => {
                let msa = refresh_msa_token(&ms_auth.refresh_token)
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                Ok(login_msa(msa).await?)
            }
        }
    }

    pub async fn logout(&self) -> Result<()> {
        Ok(())
    }

    pub fn get_username(&self) -> &str {
        match self {
            MinecraftAccount::MsaAccount { profile, .. } => &profile.name,
            MinecraftAccount::LegacyMsaAccount { name, .. } => name,
        }
    }

    pub fn get_uuid(&self) -> Uuid {
        match self {
            MinecraftAccount::MsaAccount { profile, .. } => profile.id,
            MinecraftAccount::LegacyMsaAccount { uuid, .. } => *uuid,
        }
    }
}

async fn get_device_code() -> Result<(String, String, String, u64), String> {
    let params = [
        ("client_id", AZURE_CLIENT_ID),
        ("scope", AZURE_SCOPE),
    ];
    let resp = HTTP_CLIENT
        .post(DEVICE_CODE_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Http error: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Http error: {}", e))?;

    let dc: DeviceCodeResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse device code response: {}", e))?;

    Ok((
        dc.verification_uri,
        dc.user_code,
        dc.device_code,
        dc.interval.max(2),
    ))
}

async fn poll_for_token(device_code: &str, interval: u64) -> Result<TokenResponse, String> {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;

        let params = [
            ("client_id", AZURE_CLIENT_ID),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", device_code),
        ];

        let resp = HTTP_CLIENT
            .post(TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Http error: {}", e))?;

        let status = resp.status();
        let body = resp.text().await.map_err(|e| format!("Failed to read response body: {}", e))?;

        let token: TokenResponse = serde_json::from_str(&body).map_err(|e| {
            format!(
                "Failed to parse token response (HTTP {}): {} (body: {})",
                status.as_u16(),
                e,
                &body[..body.len().min(200)]
            )
        })?;

        match token.error.as_deref() {
            None | Some("") => return Ok(token),
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
            Some(err) => {
                return Err(format!(
                    "Auth error: {} - {}",
                    err,
                    token.error_description.unwrap_or_default()
                ))
            }
        }
    }
}

async fn refresh_msa_token(
    refresh_token: &str,
) -> Result<ExpiringValue<AccessTokenResponse>, String> {
    let params = [
        ("client_id", AZURE_CLIENT_ID),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("scope", AZURE_SCOPE),
    ];

    let resp = HTTP_CLIENT
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Http error: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Http error: {}", e))?;

    let token: TokenResponse = resp.json().await
        .map_err(|e| format!("Failed to parse refresh token response: {}", e))?;

    let expires_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + token.expires_in.unwrap_or(0);

    Ok(ExpiringValue {
        data: AccessTokenResponse {
            access_token: token.access_token.unwrap_or_default(),
            refresh_token: token.refresh_token.unwrap_or_default(),
            expires_in: token.expires_in.unwrap_or(0),
            scope: AZURE_SCOPE.to_string(),
            token_type: "Bearer".to_string(),
            user_id: String::new(),
        },
        expires_at,
    })
}

async fn login_msa(msa: ExpiringValue<AccessTokenResponse>) -> Result<MinecraftAccount, AuthError> {
    let msa_token = &msa.data.access_token;
    trace!("Got access token: {msa_token}");

    let minecraft = get_minecraft_token(&HTTP_CLIENT, msa_token).await?;
    let profile = get_profile(&HTTP_CLIENT, &minecraft.minecraft_access_token).await?;

    Ok(MinecraftAccount::MsaAccount {
        msa,
        xbl: minecraft.xbl,
        mca: minecraft.mca,
        profile,
    })
}
