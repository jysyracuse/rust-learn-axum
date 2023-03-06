#[macro_use]
extern crate dotenv_codegen;
extern crate dotenv;

use dotenv::dotenv;

mod app;
mod db;
mod error;
mod routes;
mod utils;
mod middlewares;

#[tokio::main]
async fn main() {

    let app = app::create_app().await;

    // Load .env configurations
    dotenv().ok();

    // run it with hyper on $HOST:$PORT (from .env file)
    axum::Server::bind(&format!("{}:{}", dotenv!("HOST"), dotenv!("PORT")).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}