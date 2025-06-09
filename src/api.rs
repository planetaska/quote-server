//! API module for the Quotes Server.
//!
//! Provides RESTful API endpoints with OpenAPI documentation using utoipa.
//! Handles JSON responses for quotes and integrates with the database layer.
//!
use crate::{
    AppState,
    authjwt::{self, Claims, Registration},
    db::{self, CreateQuoteRequest, QuoteWithTags, UpdateQuoteRequest},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use serde::Deserialize;
use utoipa::{
    IntoParams, Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

#[derive(Debug, Deserialize, IntoParams, utoipa::ToSchema)]
pub struct SearchParams {
    /// Search within quote text
    #[param(example = "imagination")]
    pub quote: Option<String>,
    /// Search within source/author
    #[param(example = "Einstein")]
    pub source: Option<String>,
    /// Search within tags
    #[param(example = "creativity")]
    pub tag: Option<String>,
}

/// OpenAPI documentation for the Quotes API
#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_quotes,
        get_quote_by_id,
        get_random_quote,
        create_quote,
        update_quote,
        delete_quote,
        register
    ),
    components(
        schemas(QuoteWithTags, CreateQuoteRequest, UpdateQuoteRequest, Registration, authjwt::AuthBody, SearchParams)
    ),
    tags(
        (name = "quotes", description = "Quote management endpoints"),
        (name = "auth", description = "Authentication endpoints")
    ),
    info(
        title = "Quotes Server API",
        version = "0.1.0",
        description = "A simple API for managing and retrieving inspirational quotes",
        contact(
            name = "Chia-Wei Hsu",
            email = "chiawei@pdx.edu"
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        )
    }
}

/// Get all quotes from the database with optional search filters
///
/// Returns a list of quotes with their associated tags. Can be filtered by quote text, source, or tags.
#[utoipa::path(
    get,
    path = "/api/v1/quotes",
    params(SearchParams),
    responses(
        (status = 200, description = "List of quotes successfully retrieved", body = Vec<QuoteWithTags>),
        (status = 500, description = "Internal server error")
    ),
    tag = "quotes"
)]
pub async fn get_all_quotes(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Json<Vec<QuoteWithTags>> {
    let quotes = db::search_quotes(&state.pool, params)
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

/// Create a new quote (requires authentication)
///
/// Creates a new quote with optional tags and returns the created quote with its assigned ID.
#[utoipa::path(
    post,
    path = "/api/v1/quotes",
    request_body = CreateQuoteRequest,
    responses(
        (status = 201, description = "Quote successfully created", body = QuoteWithTags),
        (status = 400, description = "Invalid request body"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "quotes",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_quote(
    _claims: Claims,
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

/// Update an existing quote (requires authentication)
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
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Quote not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "quotes",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_quote(
    _claims: Claims,
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

/// Delete a quote by ID (requires authentication)
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
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Quote not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "quotes",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_quote(
    _claims: Claims,
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

/// User registration and authentication
///
/// Authenticates a user with their credentials and returns a JWT token for accessing protected endpoints.
#[utoipa::path(
    post,
    path = "/auth",
    request_body = Registration,
    responses(
        (status = 200, description = "User successfully authenticated", body = authjwt::AuthBody),
        (status = 400, description = "Invalid registration data"),
        (status = 401, description = "Wrong credentials"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(registration): Json<Registration>,
) -> axum::response::Response {
    match authjwt::make_jwt_token(&state.jwt_keys, &state.reg_key, &registration) {
        Ok(token) => (StatusCode::OK, Json(token)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create API router with all quote-related endpoints
pub fn create_api_router() -> utoipa_axum::router::OpenApiRouter<AppState> {
    utoipa_axum::router::OpenApiRouter::new()
        .route("/auth", post(register))
        .route("/api/v1/quotes", get(get_all_quotes).post(create_quote))
        .route("/api/v1/quotes/random", get(get_random_quote))
        .route(
            "/api/v1/quotes/{id}",
            get(get_quote_by_id).put(update_quote).delete(delete_quote),
        )
}
