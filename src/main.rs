mod templates;

use axum::{Router, routing::get};
use templates::{about_page, index_page};
use tower_http::trace;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn app() -> Router {
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

    // build routes
    Router::new()
        .route("/", get(index_page))
        .route("/about", get(about_page))
        .layer(trace_layer)
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // build application with routes
    let app = app();

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_main() {
        let response = app()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();

        assert_eq!(html, "<h1>Hello!</h1>");
    }
}
