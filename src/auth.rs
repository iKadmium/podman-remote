use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use std::sync::Arc;

/// Validates Bearer token against the expected token from podman secrets
pub fn validate_bearer_token(
    expected_token: Arc<String>,
) -> impl Fn(&mut Request<Body>) -> Result<(), Response<Body>> + Clone {
    move |request: &mut Request<Body>| {
        // Extract the Authorization header
        let auth_header = request
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::empty())
                    .unwrap()
            })?;

        // Check for Bearer token format
        if !auth_header.starts_with("Bearer ") {
            tracing::warn!("Authorization header missing Bearer prefix");
            return Err(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
                .unwrap());
        }

        let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
                .unwrap()
        })?;

        // Validate token against the stored token from podman secrets
        if token != expected_token.as_str() {
            tracing::warn!("Invalid authentication token attempt");
            return Err(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
                .unwrap());
        }

        tracing::debug!("Authentication successful");
        Ok(())
    }
}
