use std::time::SystemTime;

use jsonwebtoken::{
    decode, encode, errors::Error, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
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

pub fn verify_token(jwt: String) -> bool {
    debug!("Verifying JWT token {}", jwt);

    decode::<Claims>(
        jwt.as_str(),
        &DecodingKey::from_secret(MAKISHIMA_SIGKEY.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .is_ok()
}
