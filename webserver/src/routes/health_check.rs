use actix_web::{HttpResponse, Responder};
use actix_web_codegen::get;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
