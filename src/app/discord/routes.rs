use actix_web::{
    cookie::{time::OffsetDateTime, Cookie, SameSite},
    get,
    http::{header, StatusCode},
    web, HttpRequest, HttpResponse, ResponseError,
};
use log::{error, info, warn};
use sea_orm::{sea_query::OnConflict, EntityTrait, Set};

use crate::{
    app::discord::{error::DiscordError, types::DiscordOAuthTokenResponse},
    entities::{prelude::User, user},
    util::{
        jwt::{create_token, verify_token},
        oauth::{OAuthGrant, OAuthTokenRequest, OAuthTokenRevokeRequest},
    },
    AppState, DISCORD_ENDPOINT, HTTP_CLIENT, MAKISHIMA_ID, MAKISHIMA_REDIRECT, MAKISHIMA_SECRET,
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

#[get("/verify")]
pub async fn discord_verify(req: HttpRequest) -> HttpResponse {
    let cookie = req
        .cookie("identity")
        .unwrap_or(Cookie::new("identity", ""));

    let verified = verify_token(cookie.value().to_string());
    info!("Verification status: {}", verified);

    HttpResponse::new(if verified {
        StatusCode::OK
    } else {
        DiscordError::Unauthorized.status_code()
    })
}

#[get("/logout")]
pub async fn discord_logout(req: HttpRequest) -> HttpResponse {
    if let Some(identity) = req.cookie("identity") {
        let response = HTTP_CLIENT
            .post(format!(
                "{}/api/v10/oauth2/token/revoke",
                DISCORD_ENDPOINT.to_string()
            ))
            .form(&OAuthTokenRevokeRequest {
                token: identity.value().to_string(),
                token_type_hint: String::from("access_token"),
            })
            .send()
            .await;

        if let Err(err) = response {
            warn!("Token revocation request encountered a problem: {}", err);
        }
    }

    HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
        .append_header((header::LOCATION, "http://localhost:5173/"))
        .cookie(
            Cookie::build("identity", "")
                .expires(OffsetDateTime::now_utc())
                .finish(),
        )
        .finish()
}

#[get("/redirect/discord")]
pub async fn discord_oauth(
    auth: web::Query<OAuthGrant>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, DiscordError> {
    let token_exchange = OAuthTokenRequest {
        code: auth.code.clone(),
        grant_type: String::from("authorization_code"),
        redirect_uri: MAKISHIMA_REDIRECT.to_string(),
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
        .json::<DiscordOAuthTokenResponse>()
        .await
        .unwrap();

    let identity = identify_user(tokens.oauth_response.access_token.to_owned())
        .await
        .unwrap();
    let jwt = create_token(identity.to_owned(), tokens.oauth_response.expires_in);

    let user_entry = user::ActiveModel {
        id: Set(identity.id.to_owned()),
        discord_token: Set(tokens.oauth_response.access_token.to_owned()),
        anilist_id: Set(None),
    };

    if let Err(err) = User::insert(user_entry)
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
                .same_site(SameSite::Lax)
                .finish(),
        )
        .finish())
}
