use actix_web::HttpResponse;
use actix_web_codegen::get;

#[get("/health")]
pub async fn hello() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}
