use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum DiscordError {
    #[display(fmt = "internal server error")]
    InternalError,
}

impl ResponseError for DiscordError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            DiscordError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
