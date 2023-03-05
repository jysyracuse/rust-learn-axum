use axum::{
  extract::Extension,
  Router
};
use http::header;
use tower_http::{
  compression::CompressionLayer,
  cors::CorsLayer,
  propagate_header::PropagateHeaderLayer,
  sensitive_headers::SetSensitiveHeadersLayer,
  trace,
};
use tracing_subscriber::{
  // layer::SubscriberExt,
  util::SubscriberInitExt,
};
use std::sync::Arc;


use dotenv::dotenv;
// use crate::logger;
// use crate::models;

use crate::routes;
use crate::db;

pub async fn create_app() -> Router {
  // logger::setup();

  // Load .env configurations
  dotenv().ok();

  let prisma_client = Arc::new(db::new_client().await.unwrap());

  #[cfg(debug)]
  prisma_client._db_push(false).await.unwrap();

  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

  Router::new()
    .merge(routes::user::create_route())
    .merge(routes::auth::create_route())
    // .merge(Router::new().nest(
      // "/v1",
      // All public v1 routes will be nested here.
      // Router::new().merge(routes::cat::create_route()),
    // ))
    // High level logging of requests and responses
    .layer(
      trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().include_headers(true))
        .on_request(trace::DefaultOnRequest::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
    )
    // Mark the `Authorization` request header as sensitive so it doesn't
    // show in logs.
    .layer(SetSensitiveHeadersLayer::new(std::iter::once(
      header::AUTHORIZATION,
    )))
    // Compress responses
    .layer(CompressionLayer::new())
    // Propagate `X-Request-Id`s from requests to responses
    .layer(PropagateHeaderLayer::new(header::HeaderName::from_static(
      "x-request-id",
    )))
    // CORS configuration. This should probably be more restrictive in
    // production.
    .layer(CorsLayer::very_permissive())
    .layer(Extension(prisma_client))
    .layer(trace::TraceLayer::new_for_http())
}