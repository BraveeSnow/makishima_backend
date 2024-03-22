use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct DiscordAuthGrant {
    pub code: String,
}

#[derive(Serialize)]
pub struct DiscordTokenRequest {
    pub code: String,
    pub grant_type: String,
    pub redirect_uri: String,
}

#[derive(Deserialize)]
pub struct DiscordTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String,
}

#[derive(Clone, Deserialize)]
pub struct DiscordIdentity {
    pub id: String,
    pub avatar: String,
    pub global_name: String,
}
