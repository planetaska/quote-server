//! API module for the Quotes Server.
//!
//! Provides RESTful API endpoints with OpenAPI documentation using utoipa.
//! Handles JSON responses for quotes and integrates with the database layer.
//!
use crate::{db::{self, QuoteWithTags, CreateQuoteRequest}, AppState};
use axum::{extract::State, response::Json, routing::get, http::StatusCode};
use utoipa::OpenApi;

/// OpenAPI documentation for the Quotes API
#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_quotes,
        get_random_quote,
        create_quote
    ),
    components(
        schemas(QuoteWithTags, CreateQuoteRequest)
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

/// Create a new quote
///
/// Creates a new quote with optional tags and returns the created quote with its assigned ID.
#[utoipa::path(
    post,
    path = "/api/v1/quotes",
    request_body = CreateQuoteRequest,
    responses(
        (status = 201, description = "Quote successfully created", body = QuoteWithTags),
        (status = 400, description = "Invalid request body"),
        (status = 500, description = "Internal server error")
    ),
    tag = "quotes"
)]
pub async fn create_quote(
    State(state): State<AppState>,
    Json(request): Json<CreateQuoteRequest>,
) -> Result<(StatusCode, Json<QuoteWithTags>), (StatusCode, String)> {
    // Validate input
    if request.quote.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Quote text cannot be empty".to_string(),
        ));
    }

    if request.source.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Quote source cannot be empty".to_string(),
        ));
    }

    match db::create_quote(&state.pool, request).await {
        Ok(quote) => Ok((StatusCode::CREATED, Json(quote))),
        Err(err) => {
            eprintln!("Database error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create quote".to_string(),
            ))
        }
    }
}

/// Create API router with all quote-related endpoints
pub fn create_api_router() -> utoipa_axum::router::OpenApiRouter<AppState> {
    utoipa_axum::router::OpenApiRouter::new()
        .route("/api/v1/quotes", get(get_all_quotes).post(create_quote))
        .route("/api/v1/quotes/random", get(get_random_quote))
}