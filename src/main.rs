#[macro_use]
extern crate diesel;

mod operations;
mod schema;
mod models;
mod server_test;
mod config;

use actix_web::{App, HttpServer, web};
use diesel::{r2d2::{self, ConnectionManager}, sqlite::SqliteConnection};
use dotenv::dotenv;
use crate::config::ConnectionOptions;

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("Unable to locate the database.\nTry setting the 'DATABASE_URL' variable.");
    let database = Pool::builder()
        .max_size(16)
        .connection_customizer(Box::new(ConnectionOptions {
            enable_wal: true,
            enable_foreign_keys: false,
            busy_timeout: Some(std::time::Duration::from_secs(30)),
        }))
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .expect("Unable to open the database.");
    HttpServer::new(move || {
        App::new()
            .data(database.clone())
            .service(operations::get_message)
            .route("/api/message", web::post().to(operations::get_post_message))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}