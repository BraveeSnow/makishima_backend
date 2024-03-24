use serde::{Deserialize, Serialize};

use crate::util::oauth::OAuthTokenRequest;

#[derive(Serialize)]
pub struct AnilistOAuthTokenRequest {
    #[serde(flatten)]
    pub oauth: OAuthTokenRequest,
    pub client_id: String,
    pub client_secret: String,
}

// --- GRAPHQL DATA STRUCTS ---

#[derive(Deserialize)]
pub struct AnilistViewerInternal {
    pub id: u32,
}

#[derive(Deserialize)]
pub struct AnilistViewer {
    #[serde(rename = "Viewer")]
    pub viewer: AnilistViewerInternal,
}

#[derive(Deserialize)]
pub struct AnilistBaseGraphQLResponse<T> {
    pub data: T,
}
