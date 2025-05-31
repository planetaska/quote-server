//! API module for the Quotes Server.
//!
//! Provides RESTful API endpoints with OpenAPI documentation using utoipa.
//! Handles JSON responses for quotes and integrates with the database layer.
//!
use crate::{
    AppState,
    db::{self, CreateQuoteRequest, QuoteWithTags, UpdateQuoteRequest},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use utoipa::OpenApi;

/// OpenAPI documentation for the Quotes API
#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_quotes,
        get_quote_by_id,
        get_random_quote,
        create_quote,
        update_quote,
        delete_quote
    ),
    components(
        schemas(QuoteWithTags, CreateQuoteRequest, UpdateQuoteRequest)
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

/// Get a specific quote by ID
///
/// Returns a single quote with its associated tags based on the provided ID.
#[utoipa::path(
    get,
    path = "/api/v1/quotes/{id}",
    params(
        ("id" = i64, Path, description = "Quote database ID")
    ),
    responses(
        (status = 200, description = "Quote successfully retrieved", body = QuoteWithTags),
        (status = 404, description = "Quote not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "quotes"
)]
pub async fn get_quote_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<QuoteWithTags>, (StatusCode, String)> {
    match db::get_quote_by_id(&state.pool, id).await {
        Ok(Some(quote)) => Ok(Json(quote)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            format!("Quote with ID {} not found", id),
        )),
        Err(err) => {
            eprintln!("Database error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve quote".to_string(),
            ))
        }
    }
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

/// Update an existing quote
///
/// Updates an existing quote by ID with new quote text, source, and tags. All existing tags are replaced with the provided ones.
#[utoipa::path(
    put,
    path = "/api/v1/quotes/{id}",
    params(
        ("id" = i64, Path, description = "Quote database ID to update")
    ),
    request_body = UpdateQuoteRequest,
    responses(
        (status = 200, description = "Quote successfully updated", body = QuoteWithTags),
        (status = 400, description = "Invalid request body"),
        (status = 404, description = "Quote not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "quotes"
)]
pub async fn update_quote(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateQuoteRequest>,
) -> Result<Json<QuoteWithTags>, (StatusCode, String)> {
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

    match db::update_quote(&state.pool, id, request).await {
        Ok(Some(quote)) => Ok(Json(quote)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            format!("Quote with ID {} not found", id),
        )),
        Err(err) => {
            eprintln!("Database error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update quote".to_string(),
            ))
        }
    }
}

/// Delete a quote by ID
///
/// Permanently removes a quote and all its associated tags from the database.
#[utoipa::path(
    delete,
    path = "/api/v1/quotes/{id}",
    params(
        ("id" = i64, Path, description = "Quote database ID to delete")
    ),
    responses(
        (status = 204, description = "Quote successfully deleted"),
        (status = 404, description = "Quote not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "quotes"
)]
pub async fn delete_quote(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    match db::delete_quote(&state.pool, id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            format!("Quote with ID {} not found", id),
        )),
        Err(err) => {
            eprintln!("Database error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete quote".to_string(),
            ))
        }
    }
}

/// Create API router with all quote-related endpoints
pub fn create_api_router() -> utoipa_axum::router::OpenApiRouter<AppState> {
    utoipa_axum::router::OpenApiRouter::new()
        .route("/api/v1/quotes", get(get_all_quotes).post(create_quote))
        .route("/api/v1/quotes/random", get(get_random_quote))
        .route(
            "/api/v1/quotes/{id}",
            get(get_quote_by_id).put(update_quote).delete(delete_quote),
        )
}
