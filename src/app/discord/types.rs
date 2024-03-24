use serde::Deserialize;

use crate::util::oauth::OAuthTokenResponse;

#[derive(Deserialize)]
pub struct DiscordOAuthTokenResponse {
    #[serde(flatten)]
    pub oauth_response: OAuthTokenResponse,
    pub scope: String,
}

#[derive(Clone, Deserialize)]
pub struct DiscordIdentity {
    pub id: String,
    pub avatar: String,
    pub global_name: String,
}
