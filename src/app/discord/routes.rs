use std::env;

use actix_web::{
    cookie::Cookie,
    get,
    http::StatusCode,
    web::{self, Query},
    HttpResponse,
};
use log::error;
use sea_orm::{sea_query::OnConflict, EntityTrait, Set};

use crate::{
    app::discord::{
        error::DiscordError,
        types::{DiscordAuthGrant, DiscordTokenRequest, DiscordTokenResponse},
    },
    entities::user,
    util::jwt::create_token,
    AppState, DISCORD_ENDPOINT, HTTP_CLIENT, MAKISHIMA_ID, MAKISHIMA_SECRET,
};

use super::types::DiscordIdentity;

async fn identify_user(access_token: String) -> Result<DiscordIdentity, reqwest::Error> {
    let user_identity = HTTP_CLIENT
        .get(DISCORD_ENDPOINT.join("/api/v10/users/@me").unwrap())
        .bearer_auth(access_token)
        .send()
        .await;

    user_identity?.json::<DiscordIdentity>().await
}

#[get("/redirect/discord")]
pub async fn discord_oauth(
    auth: Query<DiscordAuthGrant>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, DiscordError> {
    let token_exchange = DiscordTokenRequest {
        code: auth.code.clone(),
        grant_type: String::from("authorization_code"),
        redirect_uri: env::var("MAKISHIMA_REDIRECT").unwrap(),
    };

    let token_response = HTTP_CLIENT
        .post(DISCORD_ENDPOINT.join("/api/v10/oauth2/token").unwrap())
        .form(&token_exchange)
        .basic_auth(MAKISHIMA_ID.to_string(), Some(MAKISHIMA_SECRET.to_string()))
        .send()
        .await;

    if let Err(err) = token_response {
        error!("Unable to retrieve token from Discord: {}", err);
        return Err(DiscordError::InternalError);
    }

    let tokens = token_response
        .unwrap()
        .json::<DiscordTokenResponse>()
        .await
        .unwrap();

    let identity = identify_user(tokens.access_token.to_owned()).await.unwrap();
    let jwt = create_token(identity.to_owned(), tokens.expires_in);

    let user_entry = user::ActiveModel {
        id: Set(identity.id.to_owned()),
        discord_token: Set(tokens.access_token.to_owned()),
        anilist_token: Set(None),
    };

    if let Err(err) = user::Entity::insert(user_entry)
        .on_conflict(
            OnConflict::column(user::Column::Id)
                .update_column(user::Column::DiscordToken)
                .to_owned(),
        )
        .exec(&data.db)
        .await
    {
        error!("{}", err);
        return Err(DiscordError::InternalError);
    }

    Ok(HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
        .append_header(("Location", "http://localhost:5173/"))
        .cookie(
            Cookie::build("identity", jwt.unwrap())
                .domain("localhost")
                .path("/")
                .http_only(true)
                .secure(true)
                .finish(),
        )
        .finish())
}
