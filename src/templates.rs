//! Template handling module for the Quotes Server.
//!
//! Defines Askama template structs for rendering HTML pages and implements
//! conversion from template objects into HTTP responses.
//!
use crate::AppState;
use crate::db::{self, QuoteWithTags};
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub active_page: String,
    pub quote: Option<QuoteWithTags>,
    pub has_quote: bool,
}

#[derive(Template)]
#[template(path = "about.html")]
pub struct AboutTemplate {
    pub active_page: String,
}

#[derive(Template)]
#[template(path = "quotes.html")]
pub struct QuotesTemplate {
    pub quotes: Vec<QuoteWithTags>,
    pub active_page: String,
}

#[derive(Template)]
#[template(path = "quote.html")]
pub struct QuoteTemplate {
    pub quote: Option<QuoteWithTags>,
    pub has_quote: bool,
    pub active_page: String,
}

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

pub async fn index_page(State(state): State<AppState>) -> impl IntoResponse {
    let quote = db::get_random_quote(&state.pool).await.unwrap_or(None);
    let has_quote = quote.is_some();

    let template = IndexTemplate {
        active_page: "home".to_string(),
        quote,
        has_quote,
    };
    HtmlTemplate(template)
}

pub async fn about_page() -> impl IntoResponse {
    let template = AboutTemplate {
        active_page: "about".to_string(),
    };
    HtmlTemplate(template)
}

pub async fn quotes_page(State(state): State<AppState>) -> impl IntoResponse {
    let quotes = db::get_all_quotes(&state.pool).await.unwrap_or_default();

    let template = QuotesTemplate {
        quotes,
        active_page: "quotes".to_string(),
    };
    HtmlTemplate(template)
}

pub async fn random_quote_page(State(state): State<AppState>) -> impl IntoResponse {
    let quote = db::get_random_quote(&state.pool).await.unwrap_or(None);

    let has_quote = quote.is_some();
    let template = QuoteTemplate {
        quote,
        has_quote,
        active_page: "random".to_string(),
    };
    HtmlTemplate(template)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use chrono::Utc;

    #[test]
    fn test_about_template_render() {
        // Create an about template
        let template = AboutTemplate {
            active_page: "about".to_string(),
        };

        // Test that the template can be rendered
        let result = template.render();
        assert!(result.is_ok());

        // Check that the rendered HTML contains expected content
        let html = result.unwrap();
        assert!(html.contains("<h1>About</h1>"));
    }

    #[test]
    fn test_template_into_response() {
        // Create a simple template
        let template = AboutTemplate {
            active_page: "about".to_string(),
        };

        // Create the HtmlTemplate wrapper and convert to response
        let html_template = HtmlTemplate(template);
        let response = html_template.into_response();

        // Verify response status
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[test]
    fn test_quote_template() {
        // Create a test quote
        let now = Utc::now();
        let quote = QuoteWithTags {
            id: 1,
            quote: "Test quote".to_string(),
            source: "Test source".to_string(),
            created_at: now,
            updated_at: now,
            tags: vec!["test".to_string()],
        };

        // Create a quote template with the test quote
        let template = QuoteTemplate {
            quote: Some(quote),
            has_quote: true,
            active_page: "random".to_string(),
        };

        // Test that the template can be rendered
        let result = template.render();
        assert!(result.is_ok());

        // Check that the rendered HTML contains the quote
        let html = result.unwrap();
        assert!(html.contains("Test quote"));
        assert!(html.contains("Test source"));
        assert!(html.contains("test")); // Tag should be present
    }
}
