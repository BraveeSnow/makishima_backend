use std::time::{SystemTime, UNIX_EPOCH};

use actix_web::{
    cookie::{time::OffsetDateTime, Cookie},
    get,
    http::{header, StatusCode},
    web::{Data, Query},
    HttpRequest, HttpResponse,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use log::{debug, error};
use sea_orm::{sea_query::OnConflict, EntityTrait, Set, Unchanged};

use crate::{
    app::anilist::types::AnilistOAuthTokenRequest,
    entities::{
        anilist,
        prelude::{Anilist, User},
        user,
    },
    util::{
        graphql::GraphQLRequest,
        jwt::{verify_token, Claims},
        oauth::{OAuthGrant, OAuthTokenRequest, OAuthTokenResponse},
    },
    AppState, ANILIST_ENDPOINT, ANILIST_GRAPHQL_ENDPOINT, ANILIST_ID, ANILIST_REDIRECT,
    ANILIST_SECRET, HTTP_CLIENT, MAKISHIMA_SIGKEY,
};

use super::types::{AnilistBaseGraphQLResponse, AnilistViewer};

static ANILIST_GET_IDENTITY: &str = "
query GetIdentity {
    Viewer {
        id
    }
}
";

async fn identify_anilist_user(token: &String) -> Result<u32, reqwest::Error> {
    let gql_request = GraphQLRequest {
        operation_name: String::from(""),
        query: ANILIST_GET_IDENTITY.to_string(),
        variables: None,
    };

    let request = HTTP_CLIENT
        .post(ANILIST_GRAPHQL_ENDPOINT.as_ref())
        .json(&gql_request)
        .bearer_auth(token)
        .send()
        .await;

    debug!("{}", request.unwrap().text().await.unwrap());

    Ok(HTTP_CLIENT
        .post(ANILIST_GRAPHQL_ENDPOINT.as_ref())
        .json(&gql_request)
        .bearer_auth(token)
        .send()
        .await?
        .json::<AnilistBaseGraphQLResponse<AnilistViewer>>()
        .await?
        .data
        .viewer
        .id)
}

#[get("/redirect/anilist")]
pub async fn anilist_redirect(
    req: HttpRequest,
    auth: Query<OAuthGrant>,
    data: Data<AppState>,
) -> HttpResponse {
    let jwt = req
        .cookie("identity")
        .unwrap_or(Cookie::new("identity", ""));

    if jwt.value().is_empty() || !verify_token(jwt.value().to_string()) {
        return HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
            .append_header((header::LOCATION, "http://localhost:5173/"))
            .cookie(
                Cookie::build("identity", "")
                    .expires(OffsetDateTime::now_utc())
                    .finish(),
            )
            .finish();
    }

    let token_exchange = OAuthTokenRequest {
        code: auth.code.clone(),
        grant_type: String::from("authorization_code"),
        redirect_uri: ANILIST_REDIRECT.to_string(),
    };

    let anilist_token_request = AnilistOAuthTokenRequest {
        oauth: token_exchange,
        client_id: ANILIST_ID.to_string(),
        client_secret: ANILIST_SECRET.to_string(),
    };

    let token_response = HTTP_CLIENT
        .post(ANILIST_ENDPOINT.join("/api/v2/oauth/token").unwrap())
        .header("Accept", "application/json")
        .json(&anilist_token_request)
        .send()
        .await;

    if let Err(err) = token_response {
        error!("Unable to retrieve token from Anilist: {}", err);
        return HttpResponse::new(StatusCode::UNAUTHORIZED);
    }

    let tokens = token_response.unwrap().json::<OAuthTokenResponse>().await;
    if let Err(err) = tokens {
        error!("Unable to parse Anilist token response: {}", err);
        return HttpResponse::build(StatusCode::UNAUTHORIZED).finish();
    }

    let id = identify_anilist_user(&tokens.as_ref().unwrap().access_token).await;
    if let Err(err) = id {
        error!("Unable to retrieve Anilist user ID: {}", err);
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    }

    let id_unwrapped = id.unwrap();
    let tokens_unwrapped = tokens.unwrap();

    let anilist_db_result = Anilist::insert(anilist::ActiveModel {
        id: Set(id_unwrapped.clone()),
        access_token: Set(tokens_unwrapped.access_token.to_owned()),
        refresh_token: Set(tokens_unwrapped.refresh_token.to_owned()),
        expiry: Set(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + tokens_unwrapped.expires_in),
    })
    .on_conflict(
        OnConflict::column(anilist::Column::Id)
            .update_columns(vec![
                anilist::Column::AccessToken,
                anilist::Column::RefreshToken,
                anilist::Column::Expiry,
            ])
            .to_owned(),
    )
    .exec(&data.db)
    .await;

    if let Err(err) = anilist_db_result {
        error!("Unable to insert Anilist tokens into database: {}", err);
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    }

    let jwt_decoded = decode::<Claims>(
        jwt.value(),
        &DecodingKey::from_secret(MAKISHIMA_SIGKEY.as_ref()),
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    );

    let user_db_result = User::update(user::ActiveModel {
        id: Unchanged(jwt_decoded.unwrap().claims.id),
        anilist_id: Set(Some(id_unwrapped)),
        ..Default::default()
    })
    .exec(&data.db)
    .await;

    if let Err(err) = user_db_result {
        error!("Unable to insert Anilist foreign key: {}", err);
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    }

    HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
        .append_header((header::LOCATION, "http://localhost:5173/panel"))
        .finish()
}
