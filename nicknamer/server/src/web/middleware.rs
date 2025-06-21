use axum::http::{HeaderName, HeaderValue, Request, Response};
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Layer that adds CORS headers to expose HTMX headers
#[derive(Clone, Default)]
pub struct CorsExposeLayer;

impl CorsExposeLayer {
    /// Creates a new CorsExposeLayer
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for CorsExposeLayer {
    type Service = CorsExposeService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CorsExposeService { inner }
    }
}

/// Service that adds Access-Control-Expose-Headers to responses
#[derive(Clone)]
pub struct CorsExposeService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CorsExposeService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = CorsExposeFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        CorsExposeFuture {
            future: self.inner.call(request),
        }
    }
}

pin_project! {
    /// Future that resolves to a response with CORS headers added
    pub struct CorsExposeFuture<F> {
        #[pin]
        future: F,
    }
}

impl<F, ResBody, E> Future for CorsExposeFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.future.poll(cx) {
            Poll::Ready(Ok(mut response)) => {
                // Add the Access-Control-Expose-Headers header
                response.headers_mut().insert(
                    HeaderName::from_static("access-control-expose-headers"),
                    HeaderValue::from_static("hx-retarget,hx-reswap"),
                );
                Poll::Ready(Ok(response))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
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
            .layer(CorsExposeLayer::new());

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
            .layer(CorsExposeLayer::new());

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
