//! Main module for the Quotes Server.
//!
//! Handles HTTP server setup, routing configuration, and API endpoints.
//! Initializes the database connection pool and serves both HTML templates
//! and JSON API responses with OpenAPI documentation.
//!
mod api;
mod db;
mod templates;

use api::{ApiDoc, create_api_router};
use axum::{Router, http::header::HeaderValue};
use db::init_db;
use sqlx::SqlitePool;
use std::path::PathBuf;
use templates::{about_page, index_page, quotes_page, random_quote_page};
use tower_http::{services::ServeDir, trace};
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pool: SqlitePool,
}

fn app(state: AppState) -> Router {
    // setup tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "quote-server=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    // Static file service
    let assets_path = PathBuf::from("assets/static");
    let static_files_service = ServeDir::new(assets_path);

    // Create OpenAPI router for API routes
    let (api_router, api_schema) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(create_api_router())
        .split_for_parts();

    // Configure CORS
    
    let cors = CorsLayer::new()
        .allow_origin(vec![HeaderValue::from_static("http://localhost:8080")])
        .allow_headers([axum::http::header::CONTENT_TYPE])
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ]);

    // build routes
    Router::new()
        // HTML template routes
        .route("/", axum::routing::get(index_page))
        .route("/about", axum::routing::get(about_page))
        .route("/quotes", axum::routing::get(quotes_page))
        .route("/quote/random", axum::routing::get(random_quote_page))
        // Merge API routes
        .merge(api_router)
        // OpenAPI documentation routes
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api_schema))
        // Static files
        .nest_service("/static", static_files_service)
        .with_state(state)
        .layer(cors)
        .layer(trace_layer)
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize database
    let pool = init_db().await.map_err(AppError::Database)?;
    let state = AppState { pool };

    // build application with routes
    let app = app(state);

    // run the app
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .map_err(AppError::Bind)?;

    if let Ok(addr) = listener.local_addr() {
        info!("Listening on http://{addr}/");
        info!("OpenAPI documentation available at http://{addr}/swagger-ui");
    }

    axum::serve(listener, app).await.map_err(AppError::Run)
}

#[derive(displaydoc::Display, pretty_error_debug::Debug, thiserror::Error)]
enum AppError {
    /// could not bind socket
    Bind(#[source] std::io::Error),
    /// could not run server
    Run(#[source] std::io::Error),
    /// database error
    Database(#[source] sqlx::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_routes() {
        // Initialize an in-memory database for testing
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // Insert a test quote
        sqlx::query(
            "INSERT INTO quotes (quote, source, created_at, updated_at) 
             VALUES ($1, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind("Test quote")
        .bind("Test source")
        .execute(&pool)
        .await
        .unwrap();

        // Create app state
        let state = AppState { pool };

        // Create app with test state
        let app = app(state);

        // Test root route
        let response = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test API route returns JSON
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/quotes/random")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Verify content type is JSON
        let content_type = response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.contains("application/json"));
    }

    #[test]
    fn test_app_error_display() {
        let error = AppError::Bind(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "address already in use",
        ));

        assert_eq!(format!("{}", error), "could not bind socket");
    }
}
