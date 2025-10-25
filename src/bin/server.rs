use actix_web::{middleware, web, App, HttpServer};
use prompt_compress::{api, init_optimizer};
use std::sync::{Arc, Mutex};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting prompt-compress API server...");

    // Initialize optimizer
    let optimizer = init_optimizer().expect("Failed to initialize optimizer");

    // Create shared state
    let state = web::Data::new(api::AppState {
        optimizer: Arc::new(Mutex::new(optimizer)),
        review_sessions: Arc::new(Mutex::new(std::collections::HashMap::new())),
    });

    log::info!("Server starting on http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(api::configure_routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
