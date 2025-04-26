mod db;
mod templates;

use axum::{Router, extract::State, response::Json, routing::get};
use db::{QuoteWithTags, init_db};
use sqlx::SqlitePool;
use std::path::PathBuf;
use templates::{about_page, index_page, quotes_page, random_quote_page};
use tower_http::{services::ServeDir, trace};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
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

    // build routes
    Router::new()
        .route("/", get(index_page))
        .route("/about", get(about_page))
        .route("/quotes", get(quotes_page))
        .route("/api/quotes", get(get_all_quotes))
        .route("/api/quotes/random", get(get_random_quote))
        .route("/quote/random", get(random_quote_page))
        .nest_service("/static", static_files_service)
        .with_state(state)
        .layer(trace_layer)
}

async fn get_all_quotes(State(state): State<AppState>) -> Json<Vec<QuoteWithTags>> {
    let quotes = db::get_all_quotes(&state.pool)
        .await
        .expect("Failed to get quotes");
    Json(quotes)
}

async fn get_random_quote(State(state): State<AppState>) -> Json<Option<QuoteWithTags>> {
    let quote = db::get_random_quote(&state.pool)
        .await
        .expect("Failed to get random quote");
    Json(quote)
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

    // Need to be updated to handle the database state
    #[tokio::test]
    async fn test_routes() {
        // This is a simplified test that needs to be expanded
        // to properly test with database integration
        let pool = init_db().await.unwrap();
        let state = AppState { pool };

        let app = app(state);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
