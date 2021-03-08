use actix_web::{App, HttpServer, web};
mod operations;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(operations::get_message)
            .route("/api/message", web::post().to(operations::get_post_message))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}