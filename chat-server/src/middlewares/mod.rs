mod auth;
pub use auth::verify_token;
use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    request_id::{MakeRequestId, RequestId, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{warn, Level};
use uuid::Uuid;

#[derive(Clone)]
#[allow(unused)]
struct CustomRequestId(Uuid);

const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

impl MakeRequestId for CustomRequestId {
    fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::now_v7();
        match HeaderValue::from_str(&request_id.to_string()) {
            Ok(value) => Some(RequestId::new(value)),
            Err(e) => {
                warn!("Failed to create request id: {}", e);
                None
            }
        }
    }
}

pub fn set_layer(app: Router) -> Router {
    app.layer(
        ServiceBuilder::new()
            .layer(
                ServiceBuilder::new().layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(
                            DefaultOnResponse::new()
                                .level(Level::INFO)
                                .latency_unit(LatencyUnit::Micros),
                        ),
                ),
            )
            .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
            .layer(SetRequestIdLayer::new(
                X_REQUEST_ID.clone(),
                CustomRequestId(Uuid::now_v7()),
            )),
    )
}
