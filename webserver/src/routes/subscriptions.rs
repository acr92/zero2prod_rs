use actix_web::{web, HttpResponse};
use actix_web_codegen::post;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

#[post("/subscriptions")]
pub async fn subscriptions(form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().body(format!("{:#?}", form))
}
