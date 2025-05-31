//! API module for the Quotes Server.
//!
//! Provides RESTful API endpoints with OpenAPI documentation using utoipa.
//! Handles JSON responses for quotes and integrates with the database layer.
//!
use crate::{db::{self, QuoteWithTags}, AppState};
use axum::{extract::State, response::Json, routing::get};
use utoipa::OpenApi;

/// OpenAPI documentation for the Quotes API
#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_quotes,
        get_random_quote
    ),
    components(
        schemas(QuoteWithTags)
    ),
    tags(
        (name = "quotes", description = "Quote management endpoints")
    ),
    info(
        title = "Quotes Server API",
        version = "0.1.0",
        description = "A simple API for managing and retrieving inspirational quotes",
        contact(
            name = "Chia-Wei Hsu",
            email = "chiawei@pdx.edu"
        )
    )
)]
pub struct ApiDoc;

/// Get all quotes from the database
///
/// Returns a list of all quotes with their associated tags.
#[utoipa::path(
    get,
    path = "/api/v1/quotes",
    responses(
        (status = 200, description = "List of all quotes successfully retrieved", body = Vec<QuoteWithTags>),
        (status = 500, description = "Internal server error")
    ),
    tag = "quotes"
)]
pub async fn get_all_quotes(State(state): State<AppState>) -> Json<Vec<QuoteWithTags>> {
    let quotes = db::get_all_quotes(&state.pool)
        .await
        .expect("Failed to get quotes");
    Json(quotes)
}

/// Get a random quote from the database
///
/// Returns a single random quote with its associated tags, or null if no quotes are available.
#[utoipa::path(
    get,
    path = "/api/v1/quotes/random",
    responses(
        (status = 200, description = "Random quote successfully retrieved", body = Option<QuoteWithTags>),
        (status = 500, description = "Internal server error")
    ),
    tag = "quotes"
)]
pub async fn get_random_quote(State(state): State<AppState>) -> Json<Option<QuoteWithTags>> {
    let quote = db::get_random_quote(&state.pool)
        .await
        .expect("Failed to get random quote");
    Json(quote)
}

/// Create API router with all quote-related endpoints
pub fn create_api_router() -> utoipa_axum::router::OpenApiRouter<AppState> {
    utoipa_axum::router::OpenApiRouter::new()
        .route("/api/v1/quotes", get(get_all_quotes))
        .route("/api/v1/quotes/random", get(get_random_quote))
}