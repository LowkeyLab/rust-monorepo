use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;

/// Middleware function that adds CORS headers to expose HTMX headers
pub async fn cors_expose_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    // Add the Access-Control-Expose-Headers header
    response.headers_mut().insert(
        HeaderName::from_static("access-control-expose-headers"),
        HeaderValue::from_static("hx-retarget,hx-reswap"),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::{Router, response::Response};
    use tower::ServiceExt;

    #[tokio::test]
    async fn can_add_cors_expose_header() {
        let app = Router::new()
            .route("/test", axum::routing::get(|| async { "test response" }))
            .layer(axum::middleware::from_fn(cors_expose_headers));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();
        let expose_headers = headers.get("access-control-expose-headers");
        assert_eq!(
            expose_headers,
            Some(&axum::http::HeaderValue::from_static(
                "hx-retarget,hx-reswap"
            ))
        );
    }

    #[tokio::test]
    async fn can_preserve_existing_headers_when_adding_cors() {
        async fn handler_with_custom_header() -> Response<String> {
            let mut response = Response::new("test response".to_string());
            response.headers_mut().insert(
                "custom-header",
                axum::http::HeaderValue::from_static("custom-value"),
            );
            response
        }

        let app = Router::new()
            .route(
                "/test-with-headers",
                axum::routing::get(handler_with_custom_header),
            )
            .layer(axum::middleware::from_fn(cors_expose_headers));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/test-with-headers")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();

        // Check that our CORS header was added
        let expose_headers = headers.get("access-control-expose-headers");
        assert_eq!(
            expose_headers,
            Some(&axum::http::HeaderValue::from_static(
                "hx-retarget,hx-reswap"
            ))
        );

        // Check that existing headers are preserved
        let custom_header = headers.get("custom-header");
        assert_eq!(
            custom_header,
            Some(&axum::http::HeaderValue::from_static("custom-value"))
        );
    }
}
