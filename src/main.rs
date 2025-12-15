use axum::{Router, routing::get};
use std::sync::Arc;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, TraceLayer};
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod containers;
mod services;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "podman_remote=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load API token from file (podman secret)
    let token_path =
        std::env::var("API_TOKEN_FILE").unwrap_or_else(|_| "/run/secrets/api_token".to_string());
    let api_token = std::fs::read_to_string(&token_path)
        .unwrap_or_else(|e| {
            eprintln!(
                "Warning: Failed to read API token from {}: {}. Using empty token.",
                token_path, e
            );
            String::new()
        })
        .trim()
        .to_string();

    if api_token.is_empty() {
        eprintln!("Warning: API token is empty. All authenticated requests will fail.");
    } else {
        tracing::info!("API token loaded successfully from {}", token_path);
    }

    let api_token = Arc::new(api_token);

    // Build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest(
            "/containers",
            containers::router().layer(ValidateRequestHeaderLayer::custom(
                auth::validate_bearer_token(api_token.clone()),
            )),
        )
        .nest(
            "/services",
            services::router().layer(ValidateRequestHeaderLayer::custom(
                auth::validate_bearer_token(api_token),
            )),
        )
        .layer(
            TraceLayer::new(SharedClassifier::new(ServerErrorsAsFailures::new()))
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(
                    |response: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        let status = response.status();
                        if status.is_client_error() {
                            tracing::warn!(
                                status = %status,
                                latency = ?latency,
                                "client error"
                            );
                        } else {
                            tracing::info!(
                                status = %status,
                                latency = ?latency,
                                "response"
                            );
                        }
                    },
                )
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        );

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

// Handler for the root route
async fn root() -> &'static str {
    "Hello, Podman Remote!"
}

// Handler for the health check route
async fn health() -> &'static str {
    "OK"
}
