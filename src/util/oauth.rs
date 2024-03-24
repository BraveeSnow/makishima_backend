use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct OAuthGrant {
    pub code: String,
}

#[derive(Serialize)]
pub struct OAuthTokenRequest {
    pub code: String,
    pub grant_type: String,
    pub redirect_uri: String,
}

#[derive(Clone, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct OAuthTokenRevokeRequest {
    pub token: String,
    pub token_type_hint: String,
}
