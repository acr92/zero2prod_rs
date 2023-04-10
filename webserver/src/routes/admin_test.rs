use actix_4_jwt_auth::AuthenticatedUser;
use actix_web::HttpResponse;
use actix_web_codegen::get;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FoundClaims {
    pub iss: String,
    pub sub: String,
    pub aud: Vec<String>,
}

#[tracing::instrument(name = "Admin Me Healthcheck", skip(authorization))]
#[get("/me")]
pub async fn admin_me(authorization: AuthenticatedUser<FoundClaims>) -> HttpResponse {
    let message = format!("Got token: {:#?}", authorization.claims);
    HttpResponse::Ok().body(message)
}
