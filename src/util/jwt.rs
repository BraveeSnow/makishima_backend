use std::time::SystemTime;

use jsonwebtoken::{encode, errors::Error, EncodingKey, Header};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{app::discord::types::DiscordIdentity, MAKISHIMA_SIGKEY};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub username: String,
    pub iat: u64,
    pub exp: u64,
}

pub fn create_token(identity: DiscordIdentity, expiry_time: u64) -> Result<String, Error> {
    debug!("Generating JWT token");

    let issued_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = Claims {
        id: identity.id,
        username: identity.global_name,
        iat: issued_at,
        exp: issued_at + expiry_time,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(MAKISHIMA_SIGKEY.as_ref()),
    )
}
