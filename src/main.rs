use actix_web::{web, App, HttpResponse, HttpServer, Responder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(zero2prod::routes::hello))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
