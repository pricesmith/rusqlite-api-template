use crate::config::Config;

// use crate::handlers::health::get_health;
// use crate::handlers::packet::post;
// use crate::handlers::packet::get_all;
use std::error::Error;
// use actix_cors::Cors;
// use actix_session::CookieSession;
use actix_web::{web, web::ServiceConfig, App, HttpServer, middleware, HttpResponse};

pub async fn start(config: Config) -> Result<(), Box<dyn Error>> {

    let addr = format!("{}:{}", config.ip, config.port);

    HttpServer::new(move || App::new()
        .data(config.clone()) // <- create app with shared state
        .wrap(middleware::Logger::default())
        // .configure(setup_cors)
        // .configure(setup_session_middleware)
        // .configure(setup_db)
        .configure(setup_routes))
        .bind(addr)?
        .run()
        .await?;

    Ok(())
}

fn setup_routes(cfg: &mut ServiceConfig) {

    cfg

        // Healthcheck
        .route("/health", web::get().to(|| HttpResponse::Ok()))

            // Raw Packet Routes
            .service(
                web::scope("/packets")
                    .route("", web::get().to(|| HttpResponse::Ok()))
                    .route("", web::post().to(|| HttpResponse::Ok())),
            )
            .service(
                web::resource("/packets/{id}")
                    .route(web::get().to(|| HttpResponse::Ok()))
                    .route(web::delete().to(|| HttpResponse::Ok())),
            );

            // .default_service(web::route().to(|| HttpResponse::NotFound().body("404")
}

// fn setup_cors() {}

// fn setup_session_middleware() {}