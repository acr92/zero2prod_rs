use actix_web::{web, HttpResponse};
use actix_web_codegen::get;

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm new subscriber", skip(_parameters))]
#[get("/subscriptions/confirm")]
pub async fn subscriptions_confirm(_parameters: web::Query<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
